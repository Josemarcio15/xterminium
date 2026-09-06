use russh_sftp::client::SftpSession;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SftpTransferProgress {
    pub direction: String, // "upload" | "download"
    pub file_name: String,
    pub file_path: String,
    pub transferred_bytes: u64,
    pub total_bytes: u64,
    pub percentage: f64,
    pub is_done: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub is_symlink: bool,
    pub size: u64,
    pub modified: Option<u64>,
}

pub struct ActiveSftpConnection {
    pub sftp: Arc<SftpSession>,
    pub ssh_handle: Arc<Mutex<russh::client::Handle<super::session::ClientHandler>>>,
    pub current_remote_dir: String,
}
