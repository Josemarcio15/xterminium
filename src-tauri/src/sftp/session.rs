use async_trait::async_trait;
use russh::client::{self, Handler};
use russh_keys::key::PublicKey;
use russh_sftp::client::{Config, SftpSession};
use std::sync::Arc;
use tokio::sync::Mutex;

use super::types::ActiveSftpConnection;

pub struct ClientHandler;

#[async_trait]
impl Handler for ClientHandler {
    type Error = russh::Error;

    async fn check_server_key(
        &mut self,
        _server_public_key: &PublicKey,
    ) -> Result<bool, Self::Error> {
        Ok(true)
    }
}

pub async fn connect_session(
    active_session: &Arc<Mutex<Option<ActiveSftpConnection>>>,
    host: &str,
    port: u16,
    user: &str,
    password: Option<&str>,
    key_path: Option<&str>,
) -> Result<String, String> {
    let client_config = client::Config {
        // Janela SSH de 8MB para vazão contínua em redes com latência
        window_size: 8 * 1024 * 1024,
        // O protocolo SSH (RFC 4253) define o pacote máximo de canal em 32KB (32768) ou até 64KB (TCP)
        maximum_packet_size: 32768,
        ..Default::default()
    };
    let config = Arc::new(client_config);
    let addr = format!("{}:{}", host, port);

    let mut session = client::connect(config, addr, ClientHandler)
        .await
        .map_err(|e| format!("Falha ao conectar via TCP/SSH ao host: {}", e))?;

    let mut authenticated = false;

    // 1. Tentar autenticação via SSH Agent (se disponível no sistema)
    if let Ok(mut agent) = russh_keys::agent::client::AgentClient::connect_env().await {
        if let Ok(identities) = agent.request_identities().await {
            for id in identities {
                let (returned_agent, result) = session.authenticate_future(user, id, agent).await;
                agent = returned_agent;
                if let Ok(true) = result {
                    authenticated = true;
                    break;
                }
            }
        }
    }

    // 2. Tentar chave privada específica se informada e ainda não autenticado
    if !authenticated {
        if let Some(path) = key_path {
            if let Ok(key) = russh_keys::load_secret_key(path, None) {
                if let Ok(true) = session.authenticate_publickey(user, Arc::new(key)).await {
                    authenticated = true;
                }
            }
        }
    }

    // 3. Tentar senha se informada e ainda não autenticado
    if !authenticated {
        if let Some(pass) = password {
            if let Ok(true) = session.authenticate_password(user, pass).await {
                authenticated = true;
            }
        }
    }

    if !authenticated {
        return Err("Falha na autenticação: Nenhuma chave no ssh-agent ou senha válida encontrada.".to_string());
    }

    // Abrir canal SFTP
    let channel = session
        .channel_open_session()
        .await
        .map_err(|e| format!("Erro ao abrir canal de sessão: {}", e))?;

    channel
        .request_subsystem(true, "sftp")
        .await
        .map_err(|e| format!("Erro ao solicitar subsistema sftp: {}", e))?;

    let config = Config {
        request_timeout_secs: 30,
        max_packet_len: 256 * 1024,
        max_concurrent_writes: 16,
    };

    let sftp = SftpSession::new_with_config(channel.into_stream(), config)
        .await
        .map_err(|e| format!("Erro ao inicializar sessão SFTP: {}", e))?;

    // Diretório padrão inicial (tenta ler o diretório padrão / home remoto via canonicalize ".")
    let home_dir = match sftp.canonicalize(".").await {
        Ok(res) => res,
        Err(_) => "/".to_string(),
    };

    let ssh_handle = Arc::new(Mutex::new(session));

    let mut lock = active_session.lock().await;
    *lock = Some(ActiveSftpConnection {
        sftp: Arc::new(sftp),
        ssh_handle,
        current_remote_dir: home_dir.clone(),
    });

    Ok(home_dir)
}

pub async fn disconnect_session(active_session: &Arc<Mutex<Option<ActiveSftpConnection>>>) {
    let mut lock = active_session.lock().await;
    *lock = None;
}
