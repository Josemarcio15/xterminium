use std::path::PathBuf;
use tauri::{AppHandle, State};

use super::{get_local_home_dir, list_local_directory, FileEntry, SftpState};

#[tauri::command]
pub async fn sftp_connect(
    host: String,
    port: Option<u16>,
    user: String,
    password: Option<String>,
    key_path: Option<String>,
    key_passphrase: Option<String>,
    sftp_state: State<'_, SftpState>,
) -> Result<String, String> {
    let port = port.unwrap_or(22);
    sftp_state
        .connect(
            &host,
            port,
            &user,
            password.as_deref(),
            key_path.as_deref(),
            key_passphrase.as_deref(),
        )
        .await
}

#[tauri::command]
pub async fn sftp_disconnect(sftp_state: State<'_, SftpState>) -> Result<(), String> {
    sftp_state.disconnect().await;
    Ok(())
}

#[tauri::command]
pub async fn sftp_list_remote(
    path: String,
    sftp_state: State<'_, SftpState>,
) -> Result<Vec<FileEntry>, String> {
    sftp_state.list_remote_dir(&path).await
}

#[tauri::command]
pub async fn sftp_list_local(path: Option<String>) -> Result<Vec<FileEntry>, String> {
    let target_path = match path {
        Some(p) if !p.is_empty() => PathBuf::from(p),
        _ => get_local_home_dir(),
    };
    list_local_directory(&target_path).await
}

#[tauri::command]
pub fn sftp_get_local_home() -> Result<String, String> {
    Ok(get_local_home_dir().to_string_lossy().to_string())
}

#[tauri::command]
pub async fn sftp_create_dir(
    path: String,
    sftp_state: State<'_, SftpState>,
) -> Result<(), String> {
    sftp_state.create_remote_dir(&path).await
}

#[tauri::command]
pub async fn sftp_create_file(
    path: String,
    sftp_state: State<'_, SftpState>,
) -> Result<(), String> {
    sftp_state.create_remote_file(&path).await
}

#[tauri::command]
pub async fn sftp_rename(
    old_path: String,
    new_path: String,
    sftp_state: State<'_, SftpState>,
) -> Result<(), String> {
    sftp_state.rename_remote(&old_path, &new_path).await
}

#[tauri::command]
pub async fn sftp_remove_file(
    path: String,
    sftp_state: State<'_, SftpState>,
) -> Result<(), String> {
    sftp_state.remove_remote_file(&path).await
}

#[tauri::command]
pub async fn sftp_remove_dir(
    path: String,
    sftp_state: State<'_, SftpState>,
) -> Result<(), String> {
    sftp_state.remove_remote_dir(&path).await
}

#[tauri::command]
pub async fn sftp_download_file(
    app: AppHandle,
    remote_path: String,
    local_path: String,
    sftp_state: State<'_, SftpState>,
) -> Result<(), String> {
    sftp_state.download_file(&app, &remote_path, &local_path).await
}

#[tauri::command]
pub async fn sftp_upload_file(
    app: AppHandle,
    local_path: String,
    remote_path: String,
    sftp_state: State<'_, SftpState>,
) -> Result<(), String> {
    sftp_state.upload_file(&app, &local_path, &remote_path).await
}

#[tauri::command]
pub async fn sftp_cancel_transfer(sftp_state: State<'_, SftpState>) -> Result<(), String> {
    sftp_state.cancel_active_transfer().await;
    Ok(())
}

#[tauri::command]
pub async fn sftp_calculate_local_hash(
    local_path: String,
    sftp_state: State<'_, SftpState>,
) -> Result<String, String> {
    sftp_state.calculate_local_sha256(&local_path).await
}

#[tauri::command]
pub async fn sftp_calculate_remote_hash(
    remote_path: String,
    sftp_state: State<'_, SftpState>,
) -> Result<String, String> {
    sftp_state.calculate_remote_sha256(&remote_path).await
}

#[tauri::command]
pub async fn sftp_create_local_dir(
    path: String,
    sftp_state: State<'_, SftpState>,
) -> Result<(), String> {
    sftp_state.create_local_dir(&path).await
}

#[tauri::command]
pub async fn sftp_create_local_file(
    path: String,
    sftp_state: State<'_, SftpState>,
) -> Result<(), String> {
    sftp_state.create_local_file(&path).await
}

#[tauri::command]
pub async fn sftp_rename_local(
    old_path: String,
    new_path: String,
    sftp_state: State<'_, SftpState>,
) -> Result<(), String> {
    sftp_state.rename_local(&old_path, &new_path).await
}

#[tauri::command]
pub async fn sftp_remove_local_file(
    path: String,
    sftp_state: State<'_, SftpState>,
) -> Result<(), String> {
    sftp_state.remove_local_file(&path).await
}

#[tauri::command]
pub async fn sftp_remove_local_dir(
    path: String,
    sftp_state: State<'_, SftpState>,
) -> Result<(), String> {
    sftp_state.remove_local_dir(&path).await
}

#[tauri::command]
pub async fn sftp_exec_remote_sudo(
    password: String,
    command: String,
    sftp_state: State<'_, SftpState>,
) -> Result<(), String> {
    sftp_state.exec_remote_sudo(&password, &command).await
}

#[tauri::command]
pub async fn sftp_exec_local_sudo(
    password: String,
    command: String,
    sftp_state: State<'_, SftpState>,
) -> Result<(), String> {
    sftp_state.exec_local_sudo(&password, &command).await
}
