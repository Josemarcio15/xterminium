use std::sync::Arc;
use tokio::sync::Mutex;
use russh::ChannelMsg;
use tokio::io::AsyncWriteExt;

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
            ChannelMsg::ExitStatus { exit_status: status } => {
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
            return Err(format!("Erro ao executar sudo (código {}): {}", status, cleaned_err));
        }
    }

    Ok(())
}

/// Executa um comando local com privilégios sudo
pub async fn exec_local_sudo(password: &str, command: &str) -> Result<(), String> {
    let mut child = tokio::process::Command::new("sudo")
        .arg("-S")
        .arg("-p")
        .arg("")
        .arg("sh")
        .arg("-c")
        .arg(command)
        .stdin(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .stdout(std::process::Stdio::null())
        .spawn()
        .map_err(|e| format!("Erro ao iniciar processo sudo local: {}", e))?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(format!("{}\n", password).as_bytes())
            .await
            .map_err(|e| format!("Erro ao enviar senha para sudo local: {}", e))?;
    }

    let output = child
        .wait_with_output()
        .await
        .map_err(|e| format!("Erro ao aguardar processo sudo local: {}", e))?;

    if !output.status.success() {
        let err_msg = String::from_utf8_lossy(&output.stderr);
        let cleaned_err = err_msg.trim();
        let lower = cleaned_err.to_lowercase();
        if lower.contains("incorrect password")
            || lower.contains("try again")
            || lower.contains("authentication failed")
            || lower.contains("authentication failure")
        {
            return Err("Senha sudo incorreta.".to_string());
        }
        return Err(format!("Falha no sudo local: {}", cleaned_err));
    }

    Ok(())
}
