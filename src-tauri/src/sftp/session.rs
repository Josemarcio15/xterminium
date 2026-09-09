use async_trait::async_trait;
use russh::client::{self, Handler};
use russh_keys::key::PublicKey;
use russh_sftp::client::{Config, SftpSession};
use std::sync::Arc;
use tokio::sync::Mutex;

use super::types::ActiveSftpConnection;

pub struct ClientHandler {
    pub host: String,
    pub port: u16,
}

#[async_trait]
impl Handler for ClientHandler {
    type Error = russh::Error;

    async fn check_server_key(
        &mut self,
        server_public_key: &PublicKey,
    ) -> Result<bool, Self::Error> {
        let home = super::local_fs::get_local_home_dir();
        let known_hosts_path = home.join(".ssh").join("known_hosts");

        if known_hosts_path.exists() {
            match russh_keys::check_known_hosts_path(&self.host, self.port, server_public_key, &known_hosts_path) {
                Ok(true) => {
                    log::info!("Chave do servidor SSH '{}' verificada com sucesso em known_hosts.", self.host);
                    return Ok(true);
                }
                Ok(false) => {
                    log::warn!("A chave do servidor SSH '{}' NÃO confere com a chave registrada em known_hosts!", self.host);
                    // Retorna falso para impedir conexão quando a chave diverge (possível MITM)
                    return Ok(false);
                }
                Err(err) => {
                    log::warn!("Host '{}' não encontrado em known_hosts ou erro ao ler arquivo: {:?}", self.host, err);
                }
            }
        }

        // Se known_hosts não existir ou não contiver o host ainda, permite a conexão inicial e registra log informativo
        log::info!("Primeira conexão ao host '{}' ou known_hosts ausente. Permitindo prosseguir com handshake.", self.host);
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
    key_passphrase: Option<&str>,
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

    let handler = ClientHandler {
        host: host.to_string(),
        port,
    };
    let mut session = client::connect(config, addr, handler)
        .await
        .map_err(|e| format!("Falha ao conectar via TCP/SSH ao host: {}", e))?;


    let mut authenticated = false;
    let mut key_failed_due_to_passphrase = false;

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

    // 2. Tentar chave privada específica ou chave padrão do sistema (~/.ssh/id_rsa, id_ed25519)
    if !authenticated {
        let resolved_key_path: Option<std::path::PathBuf> = if let Some(path_str) = key_path {
            if !path_str.trim().is_empty() {
                let p = path_str.trim();
                let home = super::local_fs::get_local_home_dir();
                if p.starts_with("~/") || p.starts_with("~\\") {
                    Some(home.join(&p[2..]))
                } else if p == "~" {
                    Some(home)
                } else {
                    Some(std::path::PathBuf::from(p))
                }
            } else {
                None
            }
        } else {
            None
        };

        // Se nenhuma chave foi explicitamente informada, verifica se existem as chaves padrão no diretório ~/.ssh
        let candidate_key = if let Some(p) = resolved_key_path {
            Some(p)
        } else {
            let home = super::local_fs::get_local_home_dir();
            let ssh_dir = home.join(".ssh");
            let candidates = ["id_rsa", "id_ed25519", "id_ecdsa", "id_dsa"];
            candidates
                .iter()
                .map(|c| ssh_dir.join(c))
                .find(|p| p.exists())
        };

        if let Some(path) = candidate_key {
            match russh_keys::load_secret_key(&path, key_passphrase) {
                Ok(key) => {
                    if let Ok(true) = session.authenticate_publickey(user, Arc::new(key)).await {
                        authenticated = true;
                    }
                }
                Err(err) => {
                    let err_str = format!("{:?}", err);
                    log::warn!("Erro ao carregar chave SSH '{}': {}", path.display(), err_str);
                    // Se não foi fornecida uma passphrase, qualquer erro ao decodificar/carregar
                    // indica que a chave é criptografada e precisa de passphrase.
                    if key_passphrase.is_none() {
                        key_failed_due_to_passphrase = true;
                    }
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
        if key_failed_due_to_passphrase && key_passphrase.is_none() {
            return Err("PASSPHRASE_REQUIRED: A chave privada SSH requer uma passphrase.".to_string());
        }
        return Err("AUTH_FAILED: Falha na autenticação: Chave ou senha inválida para o host.".to_string());
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
        transfer_cancel_token: Arc::new(Mutex::new(None)),
    });

    Ok(home_dir)
}

pub async fn disconnect_session(active_session: &Arc<Mutex<Option<ActiveSftpConnection>>>) {
    let mut lock = active_session.lock().await;
    if let Some(session) = lock.as_ref() {
        let token_lock = session.transfer_cancel_token.lock().await;
        if let Some(token) = token_lock.as_ref() {
            token.cancel();
        }
    }
    *lock = None;
}

