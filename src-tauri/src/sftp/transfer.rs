use std::path::Path;
use std::sync::Arc;
use std::time::Instant;
use tauri::{AppHandle, Emitter};
use tokio::fs as local_fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader, BufWriter};
use tokio::sync::Mutex;

use super::types::{ActiveSftpConnection, SftpTransferProgress};

// Buffer de 256KB por chunk
const CHUNK_SIZE: usize = 256 * 1024;
// Número de requisições concorrentes em pipeline simultâneo (saturação total do canal)
const PIPELINE_CONCURRENCY: usize = 8;
pub async fn download_file(
    active_session: &Arc<Mutex<Option<ActiveSftpConnection>>>,
    app: &AppHandle,
    remote_path: &str,
    local_path: &str,
) -> Result<(), String> {
    use std::io::Write;
    println!("📥 [SFTP DOWNLOAD REQUISITADO] Remoto: '{}' -> Local: '{}'", remote_path, local_path);
    let _ = std::io::stdout().flush();

    let (sftp, file_size) = {
        let lock = active_session.lock().await;
        let session = lock
            .as_ref()
            .ok_or_else(|| "Nenhuma conexão SFTP ativa".to_string())?;
        let meta = session
            .sftp
            .metadata(remote_path)
            .await
            .map_err(|e| format!("Erro ao ler metadados remotos: {}", e))?;
        (session.sftp.clone(), meta.size.unwrap_or(0))
    };

    let file_name = Path::new(remote_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(remote_path)
        .to_string();

    println!("📊 [SFTP DOWNLOAD] Arquivo: '{}', Tamanho total: {} bytes ({:.2} MB)", 
        file_name, file_size, file_size as f64 / (1024.0 * 1024.0));
    let _ = std::io::stdout().flush();

    // Emissão inicial de progresso
    let _ = app.emit(
        "sftp://progress",
        SftpTransferProgress {
            direction: "download".to_string(),
            file_name: file_name.clone(),
            file_path: remote_path.to_string(),
            transferred_bytes: 0,
            total_bytes: file_size,
            percentage: 0.0,
            is_done: false,
        },
    );

    // Para arquivos pequenos (< 512KB) ou sem tamanho conhecido, usa o stream com buffer de 256KB
    if file_size < 512 * 1024 {
        println!("ℹ️ [SFTP Download] Arquivo menor que 512KB ({:.2} KB). Usando modo direto bufferizado.", file_size as f64 / 1024.0);
        let _ = std::io::stdout().flush();

        let remote_file = sftp
            .open(remote_path)
            .await
            .map_err(|e| format!("Erro ao abrir arquivo remoto para leitura: {}", e))?;

        let local_file = local_fs::File::create(local_path)
            .await
            .map_err(|e| format!("Erro ao criar arquivo local: {}", e))?;

        let mut reader = BufReader::with_capacity(CHUNK_SIZE, remote_file);
        let mut writer = BufWriter::with_capacity(CHUNK_SIZE, local_file);

        let mut buffer = vec![0u8; CHUNK_SIZE];

        loop {
            let n = reader
                .read(&mut buffer)
                .await
                .map_err(|e| format!("Erro ao ler dados do servidor SFTP: {}", e))?;

            if n == 0 {
                break;
            }

            writer
                .write_all(&buffer[..n])
                .await
                .map_err(|e| format!("Erro ao gravar dados no arquivo local: {}", e))?;
        }

        writer
            .flush()
            .await
            .map_err(|e| format!("Erro ao sincronizar arquivo local: {}", e))?;
    } else {
        // Pipeline Concorrente: Pré-aloca o arquivo local com o tamanho exato
        let local_file = local_fs::File::create(local_path)
            .await
            .map_err(|e| format!("Erro ao criar arquivo local: {}", e))?;

        local_file
            .set_len(file_size)
            .await
            .map_err(|e| format!("Erro ao pré-alocar espaço em disco: {}", e))?;

        // Divide o arquivo em blocos balanceados entre N workers concorrentes em pipeline
        let workers = PIPELINE_CONCURRENCY.min(file_size.div_ceil(CHUNK_SIZE as u64) as usize);
        let part_size = file_size.div_ceil(workers as u64);

        println!(
            "🚀 [SFTP Pipeline] Iniciando download de '{}' ({:.2} MB) com {} workers concorrentes (Partição: {:.2} MB por worker)...",
            file_name,
            file_size as f64 / (1024.0 * 1024.0),
            workers,
            part_size as f64 / (1024.0 * 1024.0)
        );
        let _ = std::io::stdout().flush();

        let transferred_shared = Arc::new(std::sync::atomic::AtomicU64::new(0));
        let mut tasks = Vec::with_capacity(workers);
        let start_time = Instant::now();

        for i in 0..workers {
            let start = i as u64 * part_size;
            let end = (start + part_size).min(file_size);
            if start >= end {
                continue;
            }

            let sftp_clone = sftp.clone();
            let remote_path_owned = remote_path.to_string();
            let local_path_owned = local_path.to_string();
            let transferred_counter = transferred_shared.clone();

            tasks.push(tokio::spawn(async move {
                let mut remote_file = sftp_clone
                    .open(remote_path_owned)
                    .await
                    .map_err(|e| format!("Erro ao abrir arquivo remoto no worker {}: {}", i, e))?;

                use tokio::io::AsyncSeekExt;
                remote_file
                    .seek(std::io::SeekFrom::Start(start))
                    .await
                    .map_err(|e| format!("Erro no seek remoto worker {}: {}", i, e))?;

                let mut local_file = local_fs::OpenOptions::new()
                    .write(true)
                    .open(local_path_owned)
                    .await
                    .map_err(|e| format!("Erro ao abrir arquivo local no worker {}: {}", i, e))?;

                local_file
                    .seek(std::io::SeekFrom::Start(start))
                    .await
                    .map_err(|e| format!("Erro no seek local worker {}: {}", i, e))?;

                let mut remaining = end - start;
                let mut buf = vec![0u8; CHUNK_SIZE];

                while remaining > 0 {
                    let to_read = (remaining as usize).min(CHUNK_SIZE);
                    let n = remote_file
                        .read(&mut buf[..to_read])
                        .await
                        .map_err(|e| format!("Erro de leitura no worker {}: {}", i, e))?;

                    if n == 0 {
                        break;
                    }

                    local_file
                        .write_all(&buf[..n])
                        .await
                        .map_err(|e| format!("Erro de gravação no worker {}: {}", i, e))?;

                    transferred_counter.fetch_add(n as u64, std::sync::atomic::Ordering::Relaxed);
                    remaining = remaining.saturating_sub(n as u64);
                }

                local_file
                    .flush()
                    .await
                    .map_err(|e| format!("Erro ao dar flush no worker {}: {}", i, e))?;

                Ok::<(), String>(())
            }));
        }

        // Loop de monitoramento de progresso e log de velocidade em MB/s
        let monitor_app = app.clone();
        let monitor_file_name = file_name.clone();
        let monitor_remote_path = remote_path.to_string();
        let monitor_transferred = transferred_shared.clone();
        let monitor_done = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let monitor_done_clone = monitor_done.clone();

        let monitor_handle = tokio::spawn(async move {
            let mut last_log = Instant::now();
            let mut last_bytes: u64 = 0;

            while !monitor_done_clone.load(std::sync::atomic::Ordering::Relaxed) {
                tokio::time::sleep(std::time::Duration::from_millis(200)).await;
                let cur = monitor_transferred.load(std::sync::atomic::Ordering::Relaxed);
                let pct = if file_size > 0 {
                    (cur as f64 / file_size as f64) * 100.0
                } else {
                    100.0
                };

                // Log no terminal a cada ~1 segundo mostrando a taxa real de transferência (MB/s)
                if last_log.elapsed().as_millis() >= 1000 {
                    let delta_bytes = cur.saturating_sub(last_bytes);
                    let speed_mb_s = (delta_bytes as f64 / (1024.0 * 1024.0)) / (last_log.elapsed().as_secs_f64());
                    println!(
                        "⚡ [Pipeline SFTP] Baixado: {:.2} MB / {:.2} MB ({:.1}%) - Velocidade: {:.2} MB/s",
                        cur as f64 / (1024.0 * 1024.0),
                        file_size as f64 / (1024.0 * 1024.0),
                        pct,
                        speed_mb_s
                    );
                    let _ = std::io::stdout().flush();
                    last_log = Instant::now();
                    last_bytes = cur;
                }

                let _ = monitor_app.emit(
                    "sftp://progress",
                    SftpTransferProgress {
                        direction: "download".to_string(),
                        file_name: monitor_file_name.clone(),
                        file_path: monitor_remote_path.clone(),
                        transferred_bytes: cur,
                        total_bytes: file_size,
                        percentage: pct.min(100.0),
                        is_done: false,
                    },
                );
            }
        });

        // Aguarda todos os workers concorrentes do pipeline
        for task in tasks {
            match task.await {
                Ok(res) => res?,
                Err(e) => return Err(format!("Falha na task de pipeline: {}", e)),
            }
        }

        monitor_done.store(true, std::sync::atomic::Ordering::Relaxed);
        let _ = monitor_handle.await;

        let total_secs = start_time.elapsed().as_secs_f64();
        let avg_speed = (file_size as f64 / (1024.0 * 1024.0)) / total_secs.max(0.001);
        println!(
            "✅ [SFTP Pipeline Completo] '{}' finalizado em {:.2}s! Velocidade Média: {:.2} MB/s",
            file_name,
            total_secs,
            avg_speed
        );
        let _ = std::io::stdout().flush();
    }

    // Emissão final de conclusão 100%
    let _ = app.emit(
        "sftp://progress",
        SftpTransferProgress {
            direction: "download".to_string(),
            file_name,
            file_path: remote_path.to_string(),
            transferred_bytes: file_size,
            total_bytes: file_size,
            percentage: 100.0,
            is_done: true,
        },
    );

    Ok(())
}

