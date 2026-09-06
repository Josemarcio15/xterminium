use std::sync::Arc;
use tokio::sync::Mutex;

use super::types::{ActiveSftpConnection, FileEntry};

pub async fn list_remote_directory(
    active_session: &Arc<Mutex<Option<ActiveSftpConnection>>>,
    path: &str,
) -> Result<Vec<FileEntry>, String> {
    let lock = active_session.lock().await;
    let session = lock
        .as_ref()
        .ok_or_else(|| "Nenhuma conexão SFTP ativa".to_string())?;

    let read_dir = session
        .sftp
        .read_dir(path)
        .await
        .map_err(|e| format!("Erro ao ler diretório remoto: {}", e))?;

    let mut entries = Vec::new();

    for entry in read_dir {
        let file_name = entry.file_name();
        if file_name == "." || file_name == ".." {
            continue;
        }

        let metadata = entry.metadata();
        let is_dir = metadata.file_type().is_dir();
        let is_symlink = metadata.file_type().is_symlink();

        let full_path = if path == "/" {
            format!("/{}", file_name)
        } else if path.ends_with('/') {
            format!("{}{}", path, file_name)
        } else {
            format!("{}/{}", path, file_name)
        };

        entries.push(FileEntry {
            name: file_name,
            path: full_path,
            is_dir,
            is_symlink,
            size: metadata.size.unwrap_or(0),
            modified: metadata.mtime.map(|t| t as u64),
        });
    }

    // Ordenar: Pastas primeiro, depois ordem alfabética
    entries.sort_by(|a, b| match (a.is_dir, b.is_dir) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
    });

    Ok(entries)
}

pub async fn create_remote_directory(
    active_session: &Arc<Mutex<Option<ActiveSftpConnection>>>,
    path: &str,
) -> Result<(), String> {
    let lock = active_session.lock().await;
    let session = lock
        .as_ref()
        .ok_or_else(|| "Nenhuma conexão SFTP ativa".to_string())?;

    session
        .sftp
        .create_dir(path)
        .await
        .map_err(|e| format!("Erro ao criar pasta remota: {}", e))
}

pub async fn create_remote_file_entry(
    active_session: &Arc<Mutex<Option<ActiveSftpConnection>>>,
    path: &str,
) -> Result<(), String> {
    let lock = active_session.lock().await;
    let session = lock
        .as_ref()
        .ok_or_else(|| "Nenhuma conexão SFTP ativa".to_string())?;

    let file = session
        .sftp
        .create(path)
        .await
        .map_err(|e| format!("Erro ao criar arquivo remoto: {}", e))?;

    let _ = file.close().await;
    Ok(())
}

pub async fn rename_remote_entry(
    active_session: &Arc<Mutex<Option<ActiveSftpConnection>>>,
    old_path: &str,
    new_path: &str,
) -> Result<(), String> {
    let lock = active_session.lock().await;
    let session = lock
        .as_ref()
        .ok_or_else(|| "Nenhuma conexão SFTP ativa".to_string())?;

    session
        .sftp
        .rename(old_path, new_path)
        .await
        .map_err(|e| format!("Erro ao renomear item remoto: {}", e))
}

pub async fn remove_remote_file_entry(
    active_session: &Arc<Mutex<Option<ActiveSftpConnection>>>,
    path: &str,
) -> Result<(), String> {
    let lock = active_session.lock().await;
    let session = lock
        .as_ref()
        .ok_or_else(|| "Nenhuma conexão SFTP ativa".to_string())?;

    session
        .sftp
        .remove_file(path)
        .await
        .map_err(|e| format!("Erro ao remover arquivo remoto: {}", e))
}

pub async fn remove_remote_directory(
    active_session: &Arc<Mutex<Option<ActiveSftpConnection>>>,
    path: &str,
) -> Result<(), String> {
    let lock = active_session.lock().await;
    let session = lock
        .as_ref()
        .ok_or_else(|| "Nenhuma conexão SFTP ativa".to_string())?;

    session
        .sftp
        .remove_dir(path)
        .await
        .map_err(|e| format!("Erro ao remover pasta remota: {}", e))
}
