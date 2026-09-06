use sha2::{Digest, Sha256};
use std::sync::Arc;
use tokio::fs as local_fs;
use tokio::io::AsyncReadExt;
use tokio::sync::Mutex;

use super::types::ActiveSftpConnection;

const HASH_BUFFER_SIZE: usize = 256 * 1024;

/// Calcula o hash SHA-256 de um arquivo local
pub async fn calculate_local_sha256(local_path: &str) -> Result<String, String> {
    let mut file = local_fs::File::open(local_path)
        .await
        .map_err(|e| format!("Erro ao abrir arquivo local para SHA-256: {}", e))?;

    let mut hasher = Sha256::new();
    let mut buffer = vec![0u8; HASH_BUFFER_SIZE];

    loop {
        let n = file
            .read(&mut buffer)
            .await
            .map_err(|e| format!("Erro ao ler arquivo local para SHA-256: {}", e))?;

        if n == 0 {
            break;
        }

        hasher.update(&buffer[..n]);
    }

    let result = hasher.finalize();
    Ok(format!("{:x}", result))
}

/// Calcula o hash SHA-256 de um arquivo remoto via stream SFTP
pub async fn calculate_remote_sha256(
    active_session: &Arc<Mutex<Option<ActiveSftpConnection>>>,
    remote_path: &str,
) -> Result<String, String> {
    let sftp = {
        let lock = active_session.lock().await;
        let session = lock
            .as_ref()
            .ok_or_else(|| "Nenhuma conexão SFTP ativa".to_string())?;
        session.sftp.clone()
    };

    let mut remote_file = sftp
        .open(remote_path)
        .await
        .map_err(|e| format!("Erro ao abrir arquivo remoto para SHA-256: {}", e))?;

    let mut hasher = Sha256::new();
    let mut buffer = vec![0u8; HASH_BUFFER_SIZE];

    loop {
        let n = remote_file
            .read(&mut buffer)
            .await
            .map_err(|e| format!("Erro ao ler arquivo remoto para SHA-256: {}", e))?;

        if n == 0 {
            break;
        }

        hasher.update(&buffer[..n]);
    }

    let result = hasher.finalize();
    Ok(format!("{:x}", result))
}
