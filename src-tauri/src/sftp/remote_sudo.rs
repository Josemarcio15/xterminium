//! `sudo` **no servidor remoto**, via canal SSH.
//!
//! Fica aqui (e não em `crate::platform`) porque roda no host remoto, que é
//! POSIX independente do SO onde o xterminium está rodando: um cliente Windows
//! conectado a uma VPS Linux usa este mesmo código.
//!
//! O `sudo` da máquina local é outra história e vive em `crate::platform`
//! (`exec_local_sudo`).

use russh::ChannelMsg;
use std::sync::Arc;
use tokio::sync::Mutex;

use super::types::ActiveSftpConnection;

/// Executa um comando via SSH com privilégios sudo (fornecendo a senha de forma segura via stdin)
pub async fn exec_remote_sudo(
    active_session: &Arc<Mutex<Option<ActiveSftpConnection>>>,
    password: &str,
    command: &str,
) -> Result<(), String> {
    let ssh_handle = {
        let lock = active_session.lock().await;
        let session = lock
            .as_ref()
            .ok_or_else(|| "Nenhuma conexão SFTP ativa".to_string())?;
        session.ssh_handle.clone()
    };

    let session_lock = ssh_handle.lock().await;
    let mut channel = session_lock
        .channel_open_session()
        .await
        .map_err(|e| format!("Erro ao abrir canal SSH para sudo: {}", e))?;

    // Executa sudo com -S (lê senha do stdin) e -p '' (não emite prompt)
    let full_command = format!("sudo -S -p '' {}", command);
    channel
        .exec(true, full_command)
        .await
        .map_err(|e| format!("Erro ao disparar comando sudo via SSH: {}", e))?;

    // Envia a senha para o stdin do sudo
    channel
        .data(format!("{}\n", password).as_bytes())
        .await
        .map_err(|e| format!("Erro ao enviar senha via canal seguro: {}", e))?;

    let mut stderr_output = Vec::new();
    let mut exit_status: Option<u32> = None;

    while let Some(msg) = channel.wait().await {
        match msg {
            ChannelMsg::ExtendedData { data, ext: 1 } => {
                stderr_output.extend_from_slice(&data);
            }
            ChannelMsg::ExitStatus {
                exit_status: status,
            } => {
                exit_status = Some(status);
            }
            _ => {}
        }
    }

    if let Some(status) = exit_status {
        if status != 0 {
            let err_msg = String::from_utf8_lossy(&stderr_output);
            let cleaned_err = err_msg.trim();
            let lower = cleaned_err.to_lowercase();
            if lower.contains("incorrect password")
                || lower.contains("try again")
                || lower.contains("authentication failed")
                || lower.contains("authentication failure")
            {
                return Err("Senha sudo incorreta.".to_string());
            }
            return Err(format!(
                "Erro ao executar sudo (código {}): {}",
                status, cleaned_err
            ));
        }
    }

    Ok(())
}
