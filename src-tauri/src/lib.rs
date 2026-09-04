use portable_pty::{native_pty_system, CommandBuilder, MasterPty, PtySize};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::thread;
use tauri::{AppHandle, Emitter, State};

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
        if let Ok(output) = std::process::Command::new("xclip").args(&["-selection", "clipboard", "-o"]).output() {
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
            .args(&["-selection", "clipboard"])
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
        std::env::var("SHELL").unwrap_or_else(|_| "/bin/zsh".to_string())
    });

    let mut cmd = CommandBuilder::new(cmd_name);
    if let Some(arg_list) = args {
        for arg in arg_list {
            cmd.arg(arg);
        }
    }
    if let Ok(home) = std::env::var("HOME") {
        cmd.cwd(home);
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let pty_state = PtyState::default();

    tauri::Builder::default()
        .manage(pty_state)
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
            save_config
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


