pub mod checksum;
pub mod local_fs;
pub mod remote_fs;
pub mod session;
pub mod sudo;
pub mod transfer;
pub mod types;
pub mod upload;

pub use local_fs::{get_local_home_dir, list_local_directory};
pub use types::{ActiveSftpConnection, FileEntry, SftpTransferProgress};

use std::sync::Arc;
use tauri::AppHandle;
use tokio::sync::Mutex;

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
        session::connect_session(&self.active_session, host, port, user, password, key_path).await
    }

    /// Desconecta a sessão ativa
    pub async fn disconnect(&self) {
        session::disconnect_session(&self.active_session).await;
    }

    /// Obtém o diretório remoto atual
    pub async fn get_current_remote_dir(&self) -> Result<String, String> {
        let lock = self.active_session.lock().await;
        let session = lock
            .as_ref()
            .ok_or_else(|| "Nenhuma conexão SFTP ativa".to_string())?;
        Ok(session.current_remote_dir.clone())
    }

    /// Lista os arquivos e diretórios de um caminho no servidor remoto
    pub async fn list_remote_dir(&self, path: &str) -> Result<Vec<FileEntry>, String> {
        remote_fs::list_remote_directory(&self.active_session, path).await
    }

    /// Cria um novo diretório no servidor remoto
    pub async fn create_remote_dir(&self, path: &str) -> Result<(), String> {
        remote_fs::create_remote_directory(&self.active_session, path).await
    }

    /// Cria um arquivo vazio no servidor remoto
    pub async fn create_remote_file(&self, path: &str) -> Result<(), String> {
        remote_fs::create_remote_file_entry(&self.active_session, path).await
    }

    /// Renomeia ou move um arquivo/pasta no servidor remoto
    pub async fn rename_remote(&self, old_path: &str, new_path: &str) -> Result<(), String> {
        remote_fs::rename_remote_entry(&self.active_session, old_path, new_path).await
    }

    /// Remove um arquivo no servidor remoto
    pub async fn remove_remote_file(&self, path: &str) -> Result<(), String> {
        remote_fs::remove_remote_file_entry(&self.active_session, path).await
    }

    /// Remove um diretório no servidor remoto
    pub async fn remove_remote_dir(&self, path: &str) -> Result<(), String> {
        remote_fs::remove_remote_directory(&self.active_session, path).await
    }

    /// Download de arquivo remoto com streaming otimizado de 256KB e progresso
    pub async fn download_file(
        &self,
        app: &AppHandle,
        remote_path: &str,
        local_path: &str,
    ) -> Result<(), String> {
        transfer::download_file(&self.active_session, app, remote_path, local_path).await
    }

    /// Upload de arquivo local com streaming otimizado de 256KB e progresso
    pub async fn upload_file(
        &self,
        app: &AppHandle,
        local_path: &str,
        remote_path: &str,
    ) -> Result<(), String> {
        upload::upload_file(&self.active_session, app, local_path, remote_path).await
    }

    /// Calcula o hash SHA-256 de um arquivo local
    pub async fn calculate_local_sha256(&self, local_path: &str) -> Result<String, String> {
        checksum::calculate_local_sha256(local_path).await
    }

    /// Calcula o hash SHA-256 de um arquivo remoto
    pub async fn calculate_remote_sha256(&self, remote_path: &str) -> Result<String, String> {
        checksum::calculate_remote_sha256(&self.active_session, remote_path).await
    }

    /// Cria um novo diretório na máquina local
    pub async fn create_local_dir(&self, path: &str) -> Result<(), String> {
        local_fs::create_local_directory(path).await
    }

    /// Cria um arquivo vazio na máquina local
    pub async fn create_local_file(&self, path: &str) -> Result<(), String> {
        local_fs::create_local_file_entry(path).await
    }

    /// Renomeia ou move um arquivo/pasta na máquina local
    pub async fn rename_local(&self, old_path: &str, new_path: &str) -> Result<(), String> {
        local_fs::rename_local_entry(old_path, new_path).await
    }

    /// Remove um arquivo na máquina local
    pub async fn remove_local_file(&self, path: &str) -> Result<(), String> {
        local_fs::remove_local_file_entry(path).await
    }

    /// Remove um diretório na máquina local
    pub async fn remove_local_dir(&self, path: &str) -> Result<(), String> {
        local_fs::remove_local_directory(path).await
    }

    /// Executa comando com sudo no servidor remoto
    pub async fn exec_remote_sudo(&self, password: &str, command: &str) -> Result<(), String> {
        sudo::exec_remote_sudo(&self.active_session, password, command).await
    }

    /// Executa comando com sudo na máquina local
    pub async fn exec_local_sudo(&self, password: &str, command: &str) -> Result<(), String> {
        sudo::exec_local_sudo(password, command).await
    }
}


