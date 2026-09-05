use async_trait::async_trait;
use russh::client::{self, Handler};
use russh_keys::key::PublicKey;
use russh_sftp::client::{Config, SftpSession};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs as local_fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub is_symlink: bool,
    pub size: u64,
    pub modified: Option<u64>,
}

struct ClientHandler;

#[async_trait]
impl Handler for ClientHandler {
    type Error = russh::Error;

    async fn check_server_key(
        &mut self,
        _server_public_key: &PublicKey,
    ) -> Result<bool, Self::Error> {
        // Aceita a chave do host para conexões amigáveis (comportamento similar a auto-accept do cliente)
        Ok(true)
    }
}

pub struct ActiveSftpConnection {
    pub sftp: SftpSession,
    pub current_remote_dir: String,
}

#[derive(Default)]
pub struct SftpState {
    pub active_session: Arc<Mutex<Option<ActiveSftpConnection>>>,
}

impl SftpState {
    /// Conecta ao servidor SSH/SFTP usando SSH-Agent local ou senha
    pub async fn connect(
        &self,
        host: &str,
        port: u16,
        user: &str,
        password: Option<&str>,
        key_path: Option<&str>,
    ) -> Result<String, String> {
        let config = Arc::new(client::Config::default());
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

        let mut config = Config::default();
        config.request_timeout_secs = 60;

        let sftp = SftpSession::new_with_config(channel.into_stream(), config)
            .await
            .map_err(|e| format!("Erro ao inicializar sessão SFTP: {}", e))?;

        // Diretório padrão inicial (tenta ler o diretório padrão / home remoto via canonicalize ".")
        let home_dir = match sftp.canonicalize(".").await {
            Ok(res) => res,
            Err(_) => "/".to_string(),
        };

        let mut lock = self.active_session.lock().await;
        *lock = Some(ActiveSftpConnection {
            sftp,
            current_remote_dir: home_dir.clone(),
        });

        Ok(home_dir)
    }

    /// Desconecta a sessão ativa
    pub async fn disconnect(&self) {
        let mut lock = self.active_session.lock().await;
        *lock = None;
    }

    /// Lista itens do diretório remoto
    pub async fn list_remote_dir(&self, dir_path: &str) -> Result<Vec<FileEntry>, String> {
        let lock = self.active_session.lock().await;
        let session = lock
            .as_ref()
            .ok_or_else(|| "Nenhuma conexão SFTP ativa".to_string())?;

        let read_dir = session
            .sftp
            .read_dir(dir_path)
            .await
            .map_err(|e| format!("Erro ao ler diretório remoto '{}': {}", dir_path, e))?;

        let mut entries = Vec::new();

        for file in read_dir {
            let filename = file.file_name();
            if filename == "." || filename == ".." {
                continue;
            }

            let file_type = file.file_type();
            let is_dir = file_type.is_dir();
            let is_symlink = file_type.is_symlink();
            let metadata = file.metadata();

            let full_path = if dir_path.ends_with('/') {
                format!("{}{}", dir_path, filename)
            } else {
                format!("{}/{}", dir_path, filename)
            };

            entries.push(FileEntry {
                name: filename,
                path: full_path,
                is_dir,
                is_symlink,
                size: metadata.size.unwrap_or(0),
                modified: metadata.mtime.map(|t| t as u64),
            });
        }

        // Ordenar: Diretórios primeiro, depois em ordem alfabética insensível a maiúsculas
        entries.sort_by(|a, b| match (a.is_dir, b.is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        });

        Ok(entries)
    }

    /// Upload de arquivo local para o servidor remoto
    pub async fn upload_file(&self, local_path: &Path, remote_path: &str) -> Result<(), String> {
        let lock = self.active_session.lock().await;
        let session = lock
            .as_ref()
            .ok_or_else(|| "Nenhuma conexão SFTP ativa".to_string())?;

        let mut local_file = local_fs::File::open(local_path)
            .await
            .map_err(|e| format!("Erro ao abrir arquivo local: {}", e))?;

        let mut remote_file = session
            .sftp
            .create(remote_path)
            .await
            .map_err(|e| format!("Erro ao criar arquivo remoto: {}", e))?;

        let mut buffer = vec![0u8; 32 * 1024]; // 32KB chunk seguro conforme RFC do SFTP
        loop {
            let n = local_file
                .read(&mut buffer)
                .await
                .map_err(|e| format!("Erro lendo arquivo local: {}", e))?;
            if n == 0 {
                break;
            }
            remote_file
                .write_all(&buffer[..n])
                .await
                .map_err(|e| format!("Erro gravando arquivo remoto: {}", e))?;
        }

        remote_file
            .flush()
            .await
            .map_err(|e| format!("Erro ao sincronizar arquivo remoto: {}", e))?;

        // Fecha explicitamente o handle remoto aguardando o fechamento do servidor SSH,
        // evitando esgotar o open_handles do servidor (que causa "Limit exceeded: handle limit reached")
        remote_file
            .close()
            .await
            .map_err(|e| format!("Erro ao fechar arquivo remoto: {}", e))?;

        Ok(())
    }

    /// Download de arquivo remoto para a máquina local
    pub async fn download_file(&self, remote_path: &str, local_path: &Path) -> Result<(), String> {
        let lock = self.active_session.lock().await;
        let session = lock
            .as_ref()
            .ok_or_else(|| "Nenhuma conexão SFTP ativa".to_string())?;

        let mut remote_file = session
            .sftp
            .open(remote_path)
            .await
            .map_err(|e| format!("Erro ao abrir arquivo remoto: {}", e))?;

        let mut local_file = local_fs::File::create(local_path)
            .await
            .map_err(|e| format!("Erro ao criar arquivo local: {}", e))?;

        let mut buffer = vec![0u8; 32 * 1024]; // 32KB chunk seguro conforme RFC do SFTP
        loop {
            let n = remote_file
                .read(&mut buffer)
                .await
                .map_err(|e| format!("Erro lendo arquivo remoto: {}", e))?;
            if n == 0 {
                break;
            }
            local_file
                .write_all(&buffer[..n])
                .await
                .map_err(|e| format!("Erro gravando arquivo local: {}", e))?;
        }

        local_file
            .flush()
            .await
            .map_err(|e| format!("Erro ao sincronizar arquivo local: {}", e))?;

        // Fecha explicitamente o handle remoto aguardando confirmação do servidor
        remote_file
            .close()
            .await
            .map_err(|e| format!("Erro ao fechar arquivo remoto: {}", e))?;

        Ok(())
    }

    /// Upload de arquivo ou pasta recursivamente para o servidor remoto
    pub async fn upload_item_recursive(&self, local_path: &Path, remote_dest_dir: &str) -> Result<(), String> {
        let file_name = local_path
            .file_name()
            .ok_or_else(|| "Caminho local inválido".to_string())?
            .to_string_lossy();

        let remote_target = if remote_dest_dir.ends_with('/') {
            format!("{}{}", remote_dest_dir, file_name)
        } else {
            format!("{}/{}", remote_dest_dir, file_name)
        };

        if local_path.is_dir() {
            // Cria a pasta no remoto se não existir
            {
                let lock = self.active_session.lock().await;
                if let Some(session) = lock.as_ref() {
                    let _ = session.sftp.create_dir(&remote_target).await;
                }
            }

            // Lê as entradas locais e faz upload recursivo
            let mut entries = local_fs::read_dir(local_path)
                .await
                .map_err(|e| format!("Erro lendo diretório local: {}", e))?;

            while let Ok(Some(entry)) = entries.next_entry().await {
                let path = entry.path();
                Box::pin(self.upload_item_recursive(&path, &remote_target)).await?;
            }
        } else {
            self.upload_file(local_path, &remote_target).await?;
        }

        Ok(())
    }

    /// Download de arquivo ou pasta recursivamente para a máquina local
    pub async fn download_item_recursive(&self, remote_path: &str, local_dest_dir: &Path) -> Result<(), String> {
        let file_name = Path::new(remote_path)
            .file_name()
            .ok_or_else(|| "Caminho remoto inválido".to_string())?
            .to_string_lossy();

        let local_target = local_dest_dir.join(file_name.as_ref());

        // Verificar se é diretório lendo metadados remotos
        let is_dir = {
            let lock = self.active_session.lock().await;
            let session = lock
                .as_ref()
                .ok_or_else(|| "Nenhuma conexão SFTP ativa".to_string())?;
            match session.sftp.metadata(remote_path).await {
                Ok(meta) => meta.file_type().is_dir(),
                Err(_) => false,
            }
        };

        if is_dir {
            local_fs::create_dir_all(&local_target)
                .await
                .map_err(|e| format!("Erro criando pasta local: {}", e))?;

            let remote_entries = self.list_remote_dir(remote_path).await?;
            for item in remote_entries {
                Box::pin(self.download_item_recursive(&item.path, &local_target)).await?;
            }
        } else {
            self.download_file(remote_path, &local_target).await?;
        }

        Ok(())
    }
}

/// Utilitário para listar diretórios locais
pub async fn list_local_directory(dir_path: &Path) -> Result<Vec<FileEntry>, String> {
    let mut read_dir = local_fs::read_dir(dir_path)
        .await
        .map_err(|e| format!("Erro ao ler pasta local: {}", e))?;

    let mut entries = Vec::new();

    while let Ok(Some(entry)) = read_dir.next_entry().await {
        let metadata = match entry.metadata().await {
            Ok(m) => m,
            Err(_) => continue,
        };

        let is_dir = metadata.is_dir();
        let is_symlink = metadata.is_symlink();
        let file_name = entry.file_name().to_string_lossy().to_string();
        let full_path = entry.path().to_string_lossy().to_string();

        let modified = metadata
            .modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs());

        entries.push(FileEntry {
            name: file_name,
            path: full_path,
            is_dir,
            is_symlink,
            size: metadata.len(),
            modified,
        });
    }

    entries.sort_by(|a, b| match (a.is_dir, b.is_dir) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
    });

    Ok(entries)
}

/// Determina a pasta home local de forma segura
pub fn get_local_home_dir() -> PathBuf {
    if let Ok(home) = std::env::var("HOME") {
        PathBuf::from(home)
    } else {
        PathBuf::from(".")
    }
}
