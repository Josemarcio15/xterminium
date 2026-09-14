pub mod clipboard;
pub mod config;
pub mod cpu;
pub mod osc;
pub mod platform;
pub mod pty;
pub mod sftp;
pub mod shells;
pub mod updater;
pub mod window;

use clipboard::{read_clipboard, write_clipboard};
use config::{load_config, save_config};
use cpu::{get_cpu_temp, get_cpu_usage, CpuMonitorState};
use pty::{close_pty, get_pty_cwd, get_pty_status, resize_pty, spawn_pty, write_pty, PtyState};
use sftp::*;
use shells::list_shells;
use updater::{get_app_version, run_update_installer, update_needs_password};
use window::new_window;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let pty_state = PtyState::default();
    let sftp_state = SftpState::default();
    let cpu_state = CpuMonitorState::default();

    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .manage(pty_state)
        .manage(sftp_state)
        .manage(cpu_state)
        .invoke_handler(tauri::generate_handler![
            get_cpu_temp,
            get_cpu_usage,
            spawn_pty,
            write_pty,
            resize_pty,
            close_pty,
            get_pty_cwd,
            get_pty_status,
            list_shells,
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
            get_app_version,
            update_needs_password
        ])
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            // Deixa o container nativo da janela principal realmente transparente.
            // Os detalhes de cada SO ficam em `crate::platform`.
            use tauri::Manager;
            if let Some(window) = app.get_webview_window("main") {
                crate::platform::apply_transparency(&window);
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
