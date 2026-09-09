pub mod clipboard;
pub mod config;
pub mod pty;
pub mod sftp;
pub mod updater;
pub mod window;

use clipboard::{read_clipboard, write_clipboard};
use config::{load_config, save_config};
use pty::{close_pty, get_pty_cwd, get_pty_status, resize_pty, spawn_pty, write_pty, PtyState};
use sftp::*;
use updater::{get_app_version, run_update_installer};
use window::new_window;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let pty_state = PtyState::default();
    let sftp_state = SftpState::default();

    tauri::Builder::default()
        .manage(pty_state)
        .manage(sftp_state)
        .invoke_handler(tauri::generate_handler![
            spawn_pty,
            write_pty,
            resize_pty,
            close_pty,
            get_pty_cwd,
            get_pty_status,
            new_window,
            read_clipboard,
            write_clipboard,
            load_config,
            save_config,
            sftp_connect,
            sftp_disconnect,
            sftp_list_remote,
            sftp_list_local,
            sftp_get_local_home,
            sftp_create_dir,
            sftp_create_file,
            sftp_rename,
            sftp_remove_file,
            sftp_remove_dir,
            sftp_create_local_dir,
            sftp_create_local_file,
            sftp_rename_local,
            sftp_remove_local_file,
            sftp_remove_local_dir,
            sftp_download_file,
            sftp_upload_file,
            sftp_cancel_transfer,
            sftp_calculate_local_hash,
            sftp_calculate_remote_hash,
            sftp_exec_remote_sudo,
            sftp_exec_local_sudo,
            run_update_installer,
            get_app_version
        ])
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
