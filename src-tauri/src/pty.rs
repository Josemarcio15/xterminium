use portable_pty::{native_pty_system, CommandBuilder, MasterPty, PtySize};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::thread;
use tauri::{AppHandle, Emitter, State};

pub struct PtySession {
    pub writer: Arc<Mutex<Box<dyn Write + Send>>>,
    pub master: Arc<Mutex<Box<dyn MasterPty + Send>>>,
    pub child: Arc<Mutex<Box<dyn portable_pty::Child + Send>>>,
}

#[derive(Default)]
pub struct PtyState {
    pub sessions: Arc<Mutex<HashMap<String, PtySession>>>,
}

#[derive(Clone, serde::Serialize)]
pub struct PtyOutputPayload {
    pub id: String,
    pub data: String,
}

#[derive(Clone, serde::Serialize)]
pub struct PtyStatusInfo {
    pub cwd: String,
    pub foreground_process: Option<String>,
    pub cmdline: Option<String>,
    pub is_ssh: bool,
}

#[tauri::command]
pub fn write_pty(id: String, data: String, state: State<PtyState>) -> Result<(), String> {
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
pub fn resize_pty(id: String, cols: u16, rows: u16, state: State<PtyState>) -> Result<(), String> {
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
pub fn get_pty_status(id: String, state: State<PtyState>) -> Result<PtyStatusInfo, String> {
    let sessions = state.sessions.lock().map_err(|e| e.to_string())?;
    let mut cwd = String::new();
    let mut foreground_process: Option<String> = None;
    let mut cmdline: Option<String> = None;
    let mut is_ssh = false;

    if let Some(session) = sessions.get(&id) {
        let child = session.child.lock().map_err(|e| e.to_string())?;
        if let Some(pid) = child.process_id() {
            #[cfg(target_os = "linux")]
            {
                if let Ok(target) = std::fs::read_link(format!("/proc/{}/cwd", pid)) {
                    cwd = target.to_string_lossy().to_string();
                }

                // Procura processos filhos em execução no PTY (foreground child)
                let mut leaf_pid: Option<u32> = None;
                let task_dir = format!("/proc/{}/task", pid);
                if let Ok(entries) = std::fs::read_dir(task_dir) {
                    for entry in entries.flatten() {
                        let children_path = entry.path().join("children");
                        if let Ok(content) = std::fs::read_to_string(children_path) {
                            let pids: Vec<u32> = content
                                .split_whitespace()
                                .filter_map(|p| p.parse::<u32>().ok())
                                .collect();
                            if let Some(&last_p) = pids.last() {
                                leaf_pid = Some(last_p);
                            }
                        }
                    }
                }

                if let Some(fpid) = leaf_pid {
                    if let Ok(comm) = std::fs::read_to_string(format!("/proc/{}/comm", fpid)) {
                        let comm_clean = comm.trim().to_string();
                        if comm_clean == "ssh" {
                            is_ssh = true;
                        }
                        foreground_process = Some(comm_clean);
                    }
                    if let Ok(raw_cmd) = std::fs::read(format!("/proc/{}/cmdline", fpid)) {
                        let parsed = String::from_utf8_lossy(&raw_cmd)
                            .replace('\0', " ")
                            .trim()
                            .to_string();
                        if !parsed.is_empty() {
                            if parsed.starts_with("ssh ") || parsed == "ssh" {
                                is_ssh = true;
                            }
                            cmdline = Some(parsed);
                        }
                    }
                }
            }
        }
    }

    if cwd.is_empty() {
        if let Ok(home) = std::env::var("HOME") {
            cwd = home;
        } else if let Ok(userprofile) = std::env::var("USERPROFILE") {
            cwd = userprofile;
        }
    }

    Ok(PtyStatusInfo {
        cwd,
        foreground_process,
        cmdline,
        is_ssh,
    })
}

#[tauri::command]
pub fn get_pty_cwd(id: String, state: State<PtyState>) -> Result<String, String> {
    let sessions = state.sessions.lock().map_err(|e| e.to_string())?;
    if let Some(session) = sessions.get(&id) {
        let child = session.child.lock().map_err(|e| e.to_string())?;
        if let Some(_pid) = child.process_id() {
            #[cfg(target_os = "linux")]
            {
                if let Ok(target) = std::fs::read_link(format!("/proc/{}/cwd", _pid)) {
                    return Ok(target.to_string_lossy().to_string());
                }
            }
        }
    }
    // Fallback para home dir caso não consiga determinar
    if let Ok(home) = std::env::var("HOME") {
        return Ok(home);
    }
    if let Ok(userprofile) = std::env::var("USERPROFILE") {
        return Ok(userprofile);
    }
    Ok("".to_string())
}

#[tauri::command]
pub fn close_pty(id: String, state: State<PtyState>) -> Result<(), String> {
    let mut sessions = state.sessions.lock().map_err(|e| e.to_string())?;
    sessions.remove(&id);
    Ok(())
}

#[tauri::command]
pub fn spawn_pty(
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
    let app_handle = app.clone();
    thread::spawn(move || {
        let mut buffer = [0u8; 4096];
        while let Ok(n) = reader.read(&mut buffer) {
            if n == 0 {
                break;
            }
            let data = String::from_utf8_lossy(&buffer[..n]).to_string();
            let _ = app_handle.emit(
                "pty-out",
                PtyOutputPayload {
                    id: session_id.clone(),
                    data,
                },
            );
        }
        // Quando o processo do shell é encerrado (exit/Ctrl+D/EOF), emite pty-exit
        let _ = app_handle.emit("pty-exit", session_id);
    });

    Ok(())
}
