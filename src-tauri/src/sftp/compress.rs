//! Módulo de compactação em ZIP para arquivos locais e remotos (SFTP / SSH)

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tauri::{AppHandle, Emitter};
use tokio::sync::Mutex;
use tokio::time::timeout;
use russh::ChannelMsg;

use super::types::ActiveSftpConnection;

#[derive(Debug, Clone, serde::Serialize)]
pub struct SftpCompressProgress {
    pub file_name: String,
    pub processed_files: u64,
    pub total_files: u64,
    pub percentage: f64,
    pub is_done: bool,
}

/// Timeout máximo para compactação remota (60 segundos)
const REMOTE_COMPRESS_TIMEOUT: Duration = Duration::from_secs(60);

/// Calcula o caminho de destino do arquivo zip evitando sobrescrever arquivos existentes.
/// Se `arquivo.zip` existir, tenta `arquivo (2).zip`, `arquivo (3).zip`, etc.
pub fn get_available_zip_name(parent_dir: &str, base_name: &str, existing_names: &[String]) -> String {
    let raw_name = if base_name.ends_with('/') || base_name.ends_with('\\') {
        &base_name[..base_name.len() - 1]
    } else {
        base_name
    };

    let target_name = Path::new(raw_name)
        .file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_else(|| "archive".to_string());

    let stem = if let Some(stripped) = target_name.strip_suffix(".zip") {
        stripped.to_string()
    } else {
        target_name
    };

    let base_zip = format!("{}.zip", stem);
    if !existing_names.iter().any(|n| n.eq_ignore_ascii_case(&base_zip)) {
        return if parent_dir.ends_with('/') || parent_dir.ends_with('\\') {
            format!("{}{}", parent_dir, base_zip)
        } else {
            format!("{}/{}", parent_dir, base_zip)
        };
    }

    let mut counter = 2;
    loop {
        let candidate = format!("{} ({}).zip", stem, counter);
        if !existing_names.iter().any(|n| n.eq_ignore_ascii_case(&candidate)) {
            return if parent_dir.ends_with('/') || parent_dir.ends_with('\\') {
                format!("{}{}", parent_dir, candidate)
            } else {
                format!("{}/{}", parent_dir, candidate)
            };
        }
        counter += 1;
    }
}

/// Compacta um arquivo ou diretório local em formato ZIP
pub async fn compress_local_item(target_path: &str) -> Result<String, String> {
    let path = PathBuf::from(target_path);
    if !path.exists() {
        return Err("O item a ser compactado não existe localmente.".to_string());
    }

    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let mut existing_names = Vec::new();
    if let Ok(mut entries) = tokio::fs::read_dir(parent).await {
        while let Ok(Some(entry)) = entries.next_entry().await {
            existing_names.push(entry.file_name().to_string_lossy().to_string());
        }
    }

    let file_name = path.file_name().map(|f| f.to_string_lossy().to_string()).unwrap_or_default();
    let zip_dest = get_available_zip_name(&parent.to_string_lossy(), &file_name, &existing_names);

    #[cfg(target_os = "windows")]
    {
        // No Windows, usa o PowerShell Compress-Archive nativo
        let script = format!(
            "Compress-Archive -LiteralPath '{}' -DestinationPath '{}' -Force",
            target_path.replace("'", "''"),
            zip_dest.replace("'", "''")
        );
        let output = tokio::process::Command::new("powershell")
            .args(["-NoProfile", "-NonInteractive", "-Command", &script])
            .output()
            .await
            .map_err(|e| format!("Falha ao invocar powershell para compactação: {}", e))?;

        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Erro ao compactar localmente: {}", err));
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        // No Linux / macOS: tenta zip primeiro, depois tar se zip não existir
        let parent_str = parent.to_string_lossy().to_string();
        let zip_file_name = Path::new(&zip_dest)
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_else(|| "archive.zip".to_string());

        let res = tokio::process::Command::new("zip")
            .current_dir(&parent_str)
            .args(["-r", "-q", &zip_file_name, &file_name])
            .output()
            .await;

        match res {
            Ok(output) if output.status.success() => {},
            _ => {
                // Fallback para python3 shutil.make_archive caso zip não esteja instalado
                let py_script = format!(
                    "import shutil, sys; shutil.make_archive(sys.argv[1], 'zip', root_dir=sys.argv[2], base_dir=sys.argv[3])"
                );
                let zip_stem = zip_dest.strip_suffix(".zip").unwrap_or(&zip_dest);
                let py_res = tokio::process::Command::new("python3")
                    .args(["-c", &py_script, zip_stem, &parent_str, &file_name])
                    .output()
                    .await;

                if let Ok(py_out) = py_res {
                    if !py_out.status.success() {
                        let err = String::from_utf8_lossy(&py_out.stderr);
                        return Err(format!("Erro ao compactar arquivo: {}", err));
                    }
                } else {
                    return Err("Nenhum utilitário de compactação (zip ou python3) encontrado no sistema.".to_string());
                }
            }
        }
    }

    Ok(zip_dest)
}

/// Compacta um arquivo ou diretório remoto via canal SSH na VPS
pub async fn compress_remote_item(
    active_session: &Arc<Mutex<Option<ActiveSftpConnection>>>,
    app: &AppHandle,
    remote_path: &str,
) -> Result<String, String> {
    let (ssh_handle, sftp) = {
        let lock = active_session.lock().await;
        let session = lock
            .as_ref()
            .ok_or_else(|| "Nenhuma conexão SFTP ativa".to_string())?;
        (session.ssh_handle.clone(), session.sftp.clone())
    };

    let clean_path = remote_path.trim_end_matches('/');
    let parent_dir = match clean_path.rfind('/') {
        Some(pos) => if pos == 0 { "/" } else { &clean_path[..pos] },
        None => "/",
    };
    let file_name = match clean_path.rfind('/') {
        Some(pos) => &clean_path[pos + 1..],
        None => clean_path,
    };

    // Lê a pasta remota para ver os nomes existentes e gerar (2).zip se necessário
    let mut existing_names = Vec::new();
    if let Ok(entries) = sftp.read_dir(parent_dir).await {
        for entry in entries {
            existing_names.push(entry.file_name());
        }
    }

    let zip_dest = get_available_zip_name(parent_dir, file_name, &existing_names);
    let zip_file_name = match zip_dest.rfind('/') {
        Some(pos) => &zip_dest[pos + 1..],
        None => &zip_dest,
    };

    // Conta o total de arquivos para calcular a porcentagem real
    let total_files: u64 = {
        let mut count_channel = {
            let session_lock = ssh_handle.lock().await;
            session_lock.channel_open_session().await.ok()
        };
        let mut total = 1u64;
        if let Some(ref mut ch) = count_channel {
            let count_cmd = format!(
                "cd {} && find {} -type f 2>/dev/null | wc -l",
                shlex::try_quote(parent_dir).unwrap_or(std::borrow::Cow::Borrowed(parent_dir)),
                shlex::try_quote(file_name).unwrap_or(std::borrow::Cow::Borrowed(file_name)),
            );
            if ch.exec(true, count_cmd).await.is_ok() {
                let mut out = Vec::new();
                while let Some(msg) = ch.wait().await {
                    if let ChannelMsg::Data { data } = msg {
                        out.extend_from_slice(&data);
                    }
                }
                if let Ok(parsed) = String::from_utf8_lossy(&out).trim().parse::<u64>() {
                    if parsed > 0 {
                        total = parsed;
                    }
                }
            }
        }
        total
    };

    let _ = app.emit(
        "sftp-compress-progress",
        SftpCompressProgress {
            file_name: file_name.to_string(),
            processed_files: 0,
            total_files,
            percentage: 0.0,
            is_done: false,
        },
    );

    // Abre o canal SSH e LIBERA o lock de ssh_handle IMEDIATAMENTE
    let mut channel = {
        let session_lock = ssh_handle.lock().await;
        session_lock
            .channel_open_session()
            .await
            .map_err(|e| format!("Erro ao abrir canal SSH para compactação: {}", e))?
    };

    // Script inteligente no servidor Linux: usa zip (sem -q para ler progresso) se disponível, senão python3
    let cmd = format!(
        "cd {parent_quoted} && if command -v zip >/dev/null 2>&1; then zip -r {dest_quoted} {target_quoted}; else python3 -c \"import shutil; shutil.make_archive('{stem_escaped}', 'zip', '.', '{target_escaped}')\"; fi",
        parent_quoted = shlex::try_quote(parent_dir).unwrap_or(std::borrow::Cow::Borrowed(parent_dir)),
        dest_quoted = shlex::try_quote(zip_file_name).unwrap_or(std::borrow::Cow::Borrowed(zip_file_name)),
        target_quoted = shlex::try_quote(file_name).unwrap_or(std::borrow::Cow::Borrowed(file_name)),
        stem_escaped = zip_file_name.strip_suffix(".zip").unwrap_or(zip_file_name).replace("'", "\\'"),
        target_escaped = file_name.replace("'", "\\'")
    );

    channel
        .exec(true, cmd)
        .await
        .map_err(|e| format!("Erro ao executar comando de compactação remoto: {}", e))?;

    let execution_fut = async {
        let mut stderr_output = Vec::new();
        let mut stdout_buf = String::new();
        let mut processed_files = 0u64;
        let mut exit_status: Option<u32> = None;

        while let Some(msg) = channel.wait().await {
            match msg {
                ChannelMsg::Data { data } => {
                    stdout_buf.push_str(&String::from_utf8_lossy(&data));
                    while let Some(pos) = stdout_buf.find('\n') {
                        let line = stdout_buf[..pos].trim().to_string();
                        stdout_buf.drain(..=pos);
                        if line.starts_with("adding:") || line.starts_with("updating:") {
                            processed_files = (processed_files + 1).min(total_files);
                            let pct = (processed_files as f64 / total_files as f64) * 100.0;
                            let _ = app.emit(
                                "sftp-compress-progress",
                                SftpCompressProgress {
                                    file_name: file_name.to_string(),
                                    processed_files,
                                    total_files,
                                    percentage: pct.min(99.0),
                                    is_done: false,
                                },
                            );
                        }
                    }
                }
                ChannelMsg::ExtendedData { data, ext: 1 } => {
                    stderr_output.extend_from_slice(&data);
                }
                ChannelMsg::ExitStatus { exit_status: status } => {
                    exit_status = Some(status);
                }
                _ => {}
            }
        }

        if let Some(status) = exit_status {
            if status != 0 {
                let _ = app.emit(
                    "sftp-compress-progress",
                    SftpCompressProgress {
                        file_name: file_name.to_string(),
                        processed_files,
                        total_files,
                        percentage: 0.0,
                        is_done: true,
                    },
                );
                let err_msg = String::from_utf8_lossy(&stderr_output);
                return Err(format!("Erro ao compactar no servidor (código {}): {}", status, err_msg.trim()));
            }
        }

        let _ = app.emit(
            "sftp-compress-progress",
            SftpCompressProgress {
                file_name: file_name.to_string(),
                processed_files: total_files,
                total_files,
                percentage: 100.0,
                is_done: true,
            },
        );

        Ok(zip_dest)
    };

    match timeout(REMOTE_COMPRESS_TIMEOUT, execution_fut).await {
        Ok(res) => res,
        Err(_) => {
            let _ = channel.close().await;
            Err("Tempo limite excedido ao compactar arquivo no servidor remoto.".to_string())
        }
    }
}
