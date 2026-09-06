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

/// Upload de arquivo local para servidor remoto
pub async fn upload_file(
    active_session: &Arc<Mutex<Option<ActiveSftpConnection>>>,
    app: &AppHandle,
    local_path: &str,
    remote_path: &str,
) -> Result<(), String> {
    use std::io::Write;
    println!("📤 [SFTP UPLOAD REQUISITADO] Local: '{}' -> Remoto: '{}'", local_path, remote_path);
    let _ = std::io::stdout().flush();

    let local_meta = local_fs::metadata(local_path)
        .await
        .map_err(|e| format!("Erro ao ler metadados do arquivo local: {}", e))?;
    let file_size = local_meta.len();

    let sftp = {
        let lock = active_session.lock().await;
        let session = lock
            .as_ref()
            .ok_or_else(|| "Nenhuma conexão SFTP ativa".to_string())?;
        session.sftp.clone()
    };

    let local_file = local_fs::File::open(local_path)
        .await
        .map_err(|e| format!("Erro ao abrir arquivo local para leitura: {}", e))?;

    let remote_file = sftp
        .create(remote_path)
        .await
        .map_err(|e| format!("Erro ao criar arquivo no servidor remoto: {}", e))?;

    let mut reader = BufReader::with_capacity(CHUNK_SIZE, local_file);
    let mut writer = BufWriter::with_capacity(CHUNK_SIZE, remote_file);

    let mut buffer = vec![0u8; CHUNK_SIZE];
    let mut transferred: u64 = 0;
    let file_name = Path::new(local_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(local_path)
        .to_string();

    println!("🚀 [SFTP Upload] Iniciando upload de '{}' ({:.2} MB)...",
        file_name, file_size as f64 / (1024.0 * 1024.0));
    let _ = std::io::stdout().flush();

    let mut last_emit = Instant::now();
    let mut last_log = Instant::now();
    let mut last_bytes: u64 = 0;
    let start_time = Instant::now();

    // Emissão inicial de progresso
    let _ = app.emit(
        "sftp://progress",
        SftpTransferProgress {
            direction: "upload".to_string(),
            file_name: file_name.clone(),
            file_path: local_path.to_string(),
            transferred_bytes: 0,
            total_bytes: file_size,
            percentage: 0.0,
            is_done: false,
        },
    );

    loop {
        let n = reader
            .read(&mut buffer)
            .await
            .map_err(|e| format!("Erro ao ler dados do arquivo local: {}", e))?;

        if n == 0 {
            break;
        }

        writer
            .write_all(&buffer[..n])
            .await
            .map_err(|e| format!("Erro ao gravar dados no servidor SFTP: {}", e))?;

        transferred += n as u64;

        if last_log.elapsed().as_millis() >= 1000 {
            let delta_bytes = transferred.saturating_sub(last_bytes);
            let speed_mb_s = (delta_bytes as f64 / (1024.0 * 1024.0)) / (last_log.elapsed().as_secs_f64());
            let pct = if file_size > 0 {
                (transferred as f64 / file_size as f64) * 100.0
            } else {
                100.0
            };
            println!(
                "⚡ [Upload SFTP] Enviado: {:.2} MB / {:.2} MB ({:.1}%) - Velocidade: {:.2} MB/s",
                transferred as f64 / (1024.0 * 1024.0),
                file_size as f64 / (1024.0 * 1024.0),
                pct,
                speed_mb_s
            );
            let _ = std::io::stdout().flush();
            last_log = Instant::now();
            last_bytes = transferred;
        }

        if last_emit.elapsed().as_millis() >= 250 {
            let pct = if file_size > 0 {
                (transferred as f64 / file_size as f64) * 100.0
            } else {
                100.0
            };
            let _ = app.emit(
                "sftp://progress",
                SftpTransferProgress {
                    direction: "upload".to_string(),
                    file_name: file_name.clone(),
                    file_path: local_path.to_string(),
                    transferred_bytes: transferred,
                    total_bytes: file_size,
                    percentage: pct.min(100.0),
                    is_done: false,
                },
            );
            last_emit = Instant::now();
        }
    }

    writer
        .flush()
        .await
        .map_err(|e| format!("Erro ao sincronizar arquivo remoto: {}", e))?;

    let total_secs = start_time.elapsed().as_secs_f64();
    let avg_speed = (file_size as f64 / (1024.0 * 1024.0)) / total_secs.max(0.001);
    println!(
        "✅ [Upload SFTP Completo] '{}' finalizado em {:.2}s! Velocidade Média: {:.2} MB/s",
        file_name,
        total_secs,
        avg_speed
    );
    let _ = std::io::stdout().flush();

    // Emissão final de conclusão
    let _ = app.emit(
        "sftp://progress",
        SftpTransferProgress {
            direction: "upload".to_string(),
            file_name,
            file_path: local_path.to_string(),
            transferred_bytes: transferred,
            total_bytes: if file_size > 0 { file_size } else { transferred },
            percentage: 100.0,
            is_done: true,
        },
    );

    Ok(())
}
