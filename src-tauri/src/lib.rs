use portable_pty::{native_pty_system, CommandBuilder, MasterPty, PtySize};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use tauri::{AppHandle, Emitter, State};

pub mod sftp;
use sftp::{get_local_home_dir, list_local_directory, FileEntry, SftpState};

struct PtySession {
    writer: Arc<Mutex<Box<dyn Write + Send>>>,
    master: Arc<Mutex<Box<dyn MasterPty + Send>>>,
    child: Arc<Mutex<Box<dyn portable_pty::Child + Send>>>,
}

#[derive(Default)]
struct PtyState {
    sessions: Arc<Mutex<HashMap<String, PtySession>>>,
}

#[derive(Clone, serde::Serialize)]
struct PtyOutputPayload {
    id: String,
    data: String,
}

#[tauri::command]
fn write_pty(id: String, data: String, state: State<PtyState>) -> Result<(), String> {
    let sessions = state.sessions.lock().map_err(|e| e.to_string())?;
    if let Some(session) = sessions.get(&id) {
        let mut writer = session.writer.lock().map_err(|e| e.to_string())?;
        writer
            .write_all(data.as_bytes())
            .map_err(|e| e.to_string())?;
        writer.flush().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
fn resize_pty(id: String, cols: u16, rows: u16, state: State<PtyState>) -> Result<(), String> {
    let sessions = state.sessions.lock().map_err(|e| e.to_string())?;
    if let Some(session) = sessions.get(&id) {
        let master = session.master.lock().map_err(|e| e.to_string())?;
        master
            .resize(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
fn get_pty_cwd(id: String, state: State<PtyState>) -> Result<String, String> {
    let sessions = state.sessions.lock().map_err(|e| e.to_string())?;
    if let Some(session) = sessions.get(&id) {
        let child = session.child.lock().map_err(|e| e.to_string())?;
        if let Some(pid) = child.process_id() {
            #[cfg(target_os = "linux")]
            {
                if let Ok(target) = std::fs::read_link(format!("/proc/{}/cwd", pid)) {
                    return Ok(target.to_string_lossy().to_string());
                }
            }
        }
    }
    // Fallback para home dir caso não consiga determinar
    if let Ok(home) = std::env::var("HOME") {
        return Ok(home);
    }
    Ok("".to_string())
}

fn get_config_dir() -> std::path::PathBuf {
    if let Ok(config_home) = std::env::var("XDG_CONFIG_HOME") {
        std::path::PathBuf::from(config_home).join("xterminium")
    } else if let Ok(home) = std::env::var("HOME") {
        std::path::PathBuf::from(home).join(".config").join("xterminium")
    } else {
        std::path::PathBuf::from(".").join(".config").join("xterminium")
    }
}

#[tauri::command]
fn load_config(filename: String) -> Result<String, String> {
    let dir = get_config_dir();
    let file_path = dir.join(format!("{}.json", filename));
    if file_path.exists() {
        std::fs::read_to_string(file_path).map_err(|e| e.to_string())
    } else {
        Ok("".to_string())
    }
}

#[tauri::command]
fn save_config(filename: String, content: String) -> Result<(), String> {
    let dir = get_config_dir();
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let file_path = dir.join(format!("{}.json", filename));
    std::fs::write(file_path, content).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn read_clipboard(_app: AppHandle) -> Result<String, String> {
    // Usa o clipboard nativo do sistema ou xclip/wl-paste de forma ultra-rápida sem roundtrip no browser
    #[cfg(target_os = "linux")]
    {
        // Tenta ler via wl-paste ou xclip caso o webview trave
        if let Ok(output) = std::process::Command::new("wl-paste").output() {
            if output.status.success() {
                if let Ok(text) = String::from_utf8(output.stdout) {
                    return Ok(text);
                }
            }
        }
        if let Ok(output) = std::process::Command::new("xclip").args(["-selection", "clipboard", "-o"]).output() {
            if output.status.success() {
                if let Ok(text) = String::from_utf8(output.stdout) {
                    return Ok(text);
                }
            }
        }
    }
    Ok("".to_string())
}

#[tauri::command]
fn write_clipboard(text: String) -> Result<(), String> {
    #[cfg(target_os = "linux")]
    {
        use std::process::Stdio;
        if let Ok(mut child) = std::process::Command::new("wl-copy")
            .stdin(Stdio::piped())
            .spawn()
        {
            if let Some(mut stdin) = child.stdin.take() {
                let _ = stdin.write_all(text.as_bytes());
            }
            let _ = child.wait();
            return Ok(());
        }

        if let Ok(mut child) = std::process::Command::new("xclip")
            .args(["-selection", "clipboard"])
            .stdin(Stdio::piped())
            .spawn()
        {
            if let Some(mut stdin) = child.stdin.take() {
                let _ = stdin.write_all(text.as_bytes());
            }
            let _ = child.wait();
            return Ok(());
        }
    }
    Ok(())
}

#[tauri::command]
fn close_pty(id: String, state: State<PtyState>) -> Result<(), String> {
    let mut sessions = state.sessions.lock().map_err(|e| e.to_string())?;
    sessions.remove(&id);
    Ok(())
}

#[tauri::command]
fn spawn_pty(
    app: AppHandle,
    id: String,
    cols: u16,
    rows: u16,
    command: Option<String>,
    args: Option<Vec<String>>,
    state: State<PtyState>,
) -> Result<(), String> {
    let pty_system = native_pty_system();
    let pair = pty_system
        .openpty(PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        })
        .map_err(|e| e.to_string())?;

    let cmd_name = command.unwrap_or_else(|| {
        #[cfg(target_os = "windows")]
        {
            // No Windows: prioriza PowerShell, com fallback para CMD
            if let Ok(system_root) = std::env::var("SystemRoot") {
                let ps_path = format!("{}\\System32\\WindowsPowerShell\\v1.0\\powershell.exe", system_root);
                if std::path::Path::new(&ps_path).exists() {
                    return ps_path;
                }
            }
            "powershell.exe".to_string()
        }
        #[cfg(not(target_os = "windows"))]
        {
            // No Linux / macOS:
            // 1. Respeita a variável $SHELL do usuário se o binário existir
            if let Ok(user_shell) = std::env::var("SHELL") {
                if !user_shell.is_empty() && std::path::Path::new(&user_shell).exists() {
                    return user_shell;
                }
            }
            // 2. Se não existir, tenta encontrar o Zsh
            let zsh_candidates = ["/bin/zsh", "/usr/bin/zsh", "/usr/local/bin/zsh"];
            for candidate in &zsh_candidates {
                if std::path::Path::new(candidate).exists() {
                    return candidate.to_string();
                }
            }
            // 3. Fallback para Bash
            let bash_candidates = ["/bin/bash", "/usr/bin/bash"];
            for candidate in &bash_candidates {
                if std::path::Path::new(candidate).exists() {
                    return candidate.to_string();
                }
            }
            // 4. Fallback final para sh padrão POSIX
            "/bin/sh".to_string()
        }
    });

    let mut cmd = CommandBuilder::new(cmd_name);
    if let Some(arg_list) = args {
        for arg in arg_list {
            cmd.arg(arg);
        }
    }
    
    // Diretório inicial: HOME no Linux/macOS ou USERPROFILE no Windows
    if let Ok(home) = std::env::var("HOME") {
        cmd.cwd(home);
    } else if let Ok(user_profile) = std::env::var("USERPROFILE") {
        cmd.cwd(user_profile);
    }

    // Define variáveis de ambiente essenciais para o terminal reconhecer cores e comandos como clear
    cmd.env("TERM", "xterm-256color");
    cmd.env("COLORTERM", "truecolor");

    let child = pair.slave.spawn_command(cmd).map_err(|e| e.to_string())?;

    let writer = pair.master.take_writer().map_err(|e| e.to_string())?;
    let mut reader = pair.master.try_clone_reader().map_err(|e| e.to_string())?;

    let session = PtySession {
        writer: Arc::new(Mutex::new(writer)),
        master: Arc::new(Mutex::new(pair.master)),
        child: Arc::new(Mutex::new(child)),
    };

    state
        .sessions
        .lock()
        .map_err(|e| e.to_string())?
        .insert(id.clone(), session);

    let session_id = id.clone();
    thread::spawn(move || {
        let mut buffer = [0u8; 4096];
        while let Ok(n) = reader.read(&mut buffer) {
            if n == 0 {
                break;
            }
            let data = String::from_utf8_lossy(&buffer[..n]).to_string();
            let _ = app.emit(
                "pty-out",
                PtyOutputPayload {
                    id: session_id.clone(),
                    data,
                },
            );
        }
    });

    Ok(())
}

#[tauri::command]
fn new_window(app: AppHandle) -> Result<(), String> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let id = format!("win-{}", now);
    tauri::WebviewWindowBuilder::new(&app, id, tauri::WebviewUrl::default())
        .title("xterminium")
        .inner_size(900.0, 620.0)
        .decorations(false)
        .build()
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn sftp_connect(
    host: String,
    port: Option<u16>,
    user: String,
    password: Option<String>,
    key_path: Option<String>,
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
        )
        .await
}

#[tauri::command]
async fn sftp_disconnect(sftp_state: State<'_, SftpState>) -> Result<(), String> {
    sftp_state.disconnect().await;
    Ok(())
}

#[tauri::command]
async fn sftp_list_remote(
    path: String,
    sftp_state: State<'_, SftpState>,
) -> Result<Vec<FileEntry>, String> {
    sftp_state.list_remote_dir(&path).await
}

#[tauri::command]
async fn sftp_list_local(path: Option<String>) -> Result<Vec<FileEntry>, String> {
    let target_path = match path {
        Some(p) if !p.is_empty() => PathBuf::from(p),
        _ => get_local_home_dir(),
    };
    list_local_directory(&target_path).await
}

#[tauri::command]
fn sftp_get_local_home() -> Result<String, String> {
    Ok(get_local_home_dir().to_string_lossy().to_string())
}

#[tauri::command]
async fn sftp_create_dir(
    path: String,
    sftp_state: State<'_, SftpState>,
) -> Result<(), String> {
    sftp_state.create_remote_dir(&path).await
}

#[tauri::command]
async fn sftp_create_file(
    path: String,
    sftp_state: State<'_, SftpState>,
) -> Result<(), String> {
    sftp_state.create_remote_file(&path).await
}

#[tauri::command]
async fn sftp_rename(
    old_path: String,
    new_path: String,
    sftp_state: State<'_, SftpState>,
) -> Result<(), String> {
    sftp_state.rename_remote(&old_path, &new_path).await
}

#[tauri::command]
async fn sftp_remove_file(
    path: String,
    sftp_state: State<'_, SftpState>,
) -> Result<(), String> {
    sftp_state.remove_remote_file(&path).await
}

#[tauri::command]
async fn sftp_remove_dir(
    path: String,
    sftp_state: State<'_, SftpState>,
) -> Result<(), String> {
    sftp_state.remove_remote_dir(&path).await
}

#[tauri::command]
async fn sftp_download_file(
    app: AppHandle,
    remote_path: String,
    local_path: String,
    sftp_state: State<'_, SftpState>,
) -> Result<(), String> {
    sftp_state.download_file(&app, &remote_path, &local_path).await
}

#[tauri::command]
async fn sftp_upload_file(
    app: AppHandle,
    local_path: String,
    remote_path: String,
    sftp_state: State<'_, SftpState>,
) -> Result<(), String> {
    sftp_state.upload_file(&app, &local_path, &remote_path).await
}

#[tauri::command]
async fn sftp_calculate_local_hash(
    local_path: String,
    sftp_state: State<'_, SftpState>,
) -> Result<String, String> {
    sftp_state.calculate_local_sha256(&local_path).await
}

#[tauri::command]
async fn sftp_calculate_remote_hash(
    remote_path: String,
    sftp_state: State<'_, SftpState>,
) -> Result<String, String> {
    sftp_state.calculate_remote_sha256(&remote_path).await
}

#[tauri::command]
async fn sftp_create_local_dir(
    path: String,
    sftp_state: State<'_, SftpState>,
) -> Result<(), String> {
    sftp_state.create_local_dir(&path).await
}

#[tauri::command]
async fn sftp_create_local_file(
    path: String,
    sftp_state: State<'_, SftpState>,
) -> Result<(), String> {
    sftp_state.create_local_file(&path).await
}

#[tauri::command]
async fn sftp_rename_local(
    old_path: String,
    new_path: String,
    sftp_state: State<'_, SftpState>,
) -> Result<(), String> {
    sftp_state.rename_local(&old_path, &new_path).await
}

#[tauri::command]
async fn sftp_remove_local_file(
    path: String,
    sftp_state: State<'_, SftpState>,
) -> Result<(), String> {
    sftp_state.remove_local_file(&path).await
}

#[tauri::command]
async fn sftp_remove_local_dir(
    path: String,
    sftp_state: State<'_, SftpState>,
) -> Result<(), String> {
    sftp_state.remove_local_dir(&path).await
}

#[tauri::command]
async fn sftp_exec_remote_sudo(
    password: String,
    command: String,
    sftp_state: State<'_, SftpState>,
) -> Result<(), String> {
    sftp_state.exec_remote_sudo(&password, &command).await
}

#[tauri::command]
async fn sftp_exec_local_sudo(
    password: String,
    command: String,
    sftp_state: State<'_, SftpState>,
) -> Result<(), String> {
    sftp_state.exec_local_sudo(&password, &command).await
}

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
            sftp_calculate_local_hash,
            sftp_calculate_remote_hash,
            sftp_exec_remote_sudo,
            sftp_exec_local_sudo
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


