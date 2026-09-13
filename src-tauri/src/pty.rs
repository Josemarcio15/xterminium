use portable_pty::{native_pty_system, CommandBuilder, MasterPty, PtySize};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::thread;
use tauri::{AppHandle, Emitter, State};

use crate::platform::{process_cwd, pty_foreground_process};

pub struct PtySession {
    pub writer: Arc<Mutex<Box<dyn Write + Send>>>,
    pub master: Arc<Mutex<Box<dyn MasterPty + Send>>>,
    pub child: Arc<Mutex<Box<dyn portable_pty::Child + Send>>>,
    /// Último diretório reportado pelo próprio shell (OSC 9;9 / OSC 7).
    ///
    /// É a fonte mais confiável: o cwd do processo é inútil no PowerShell, que
    /// não chama `chdir` no `Set-Location`.
    pub reported_cwd: Arc<Mutex<Option<String>>>,
}

/// Wrapper de `prompt` que reporta o diretório atual via OSC 9;9.
///
/// O `-Command` do PowerShell roda **depois** dos profiles, então o prompt do
/// usuário (oh-my-posh, PSReadLine, tema custom) já existe e é preservado: nós
/// só o envolvemos. `ProviderPath` garante caminho absoluto e a checagem de
/// provider evita reportar locais que não são do filesystem (ex.: `HKLM:`).
const PROMPT_CWD_REPORT: &str = concat!(
    "$global:__xtOrig = if (Test-Path function:prompt) { (Get-Item function:prompt).ScriptBlock } else { $null }; ",
    "function global:prompt { ",
    "$p = $ExecutionContext.SessionState.Path.CurrentLocation; ",
    "if ($p.Provider.Name -eq 'FileSystem') { ",
    "[Console]::Write(([char]27) + ']9;9;' + $p.ProviderPath + ([char]7)) ",
    "}; ",
    "if ($global:__xtOrig) { & $global:__xtOrig } else { 'PS ' + $p.Path + '> ' } ",
    "}",
);

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

/// Diretório atual informado pelo shell via OSC (9;9 ou 7), se houver.
fn shell_reported_cwd(session: &PtySession) -> Option<String> {
    let slot = session.reported_cwd.lock().ok()?;
    slot.as_ref().filter(|cwd| !cwd.is_empty()).cloned()
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
        // O cwd reportado pelo shell ganha do cwd do processo: no PowerShell o
        // processo nunca muda de diretório depois do lançamento.
        if let Some(dir) = shell_reported_cwd(session) {
            cwd = dir;
        }

        let child = session.child.lock().map_err(|e| e.to_string())?;
        if let Some(pid) = child.process_id() {
            if cwd.is_empty() {
                if let Some(dir) = process_cwd(pid) {
                    cwd = dir;
                }
            }

            if let Some(fg) = pty_foreground_process(pid) {
                is_ssh = fg.is_ssh;
                cmdline = fg.cmdline;
                if !fg.name.is_empty() {
                    foreground_process = Some(fg.name);
                }
            }
        }
    }

    if cwd.is_empty() {
        cwd = crate::platform::home_dir().to_string_lossy().to_string();
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
        if let Some(dir) = shell_reported_cwd(session) {
            return Ok(dir);
        }

        let child = session.child.lock().map_err(|e| e.to_string())?;
        if let Some(pid) = child.process_id() {
            if let Some(dir) = process_cwd(pid) {
                return Ok(dir);
            }
        }
    }
    // Fallback para home dir caso não consiga determinar
    Ok(crate::platform::home_dir().to_string_lossy().to_string())
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

    // Sem preferência do usuário: usa o shell padrão do sistema.
    // (A escolha feita nas configurações chega pelo parâmetro `command`.)
    let cmd_name = command.unwrap_or_else(crate::platform::default_shell_path);
    // Não injeta se o usuário já configurou `-Command`/`-File` no shell:
    // dois `-Command` no mesmo pwsh entram em conflito.
    let args_conflict = args.as_ref().is_some_and(|list| {
        list.iter().any(|arg| {
            let flag = arg.trim_start_matches('-').to_ascii_lowercase();
            flag.starts_with("command") || flag.starts_with("file")
        })
    });
    let is_powershell = crate::shells::is_powershell(&cmd_name) && !args_conflict;

    let mut cmd = CommandBuilder::new(cmd_name);
    if let Some(arg_list) = args {
        for arg in arg_list {
            cmd.arg(arg);
        }
    }

    if is_powershell {
        cmd.arg("-NoExit");
        cmd.arg("-Command");
        cmd.arg(PROMPT_CWD_REPORT);
    }

    // Diretório inicial: home do usuário (detalhe por SO em `crate::platform`)
    cmd.cwd(crate::platform::home_dir());

    // Define variáveis de ambiente essenciais para o terminal reconhecer cores e comandos como clear
    cmd.env("TERM", "xterm-256color");
    cmd.env("COLORTERM", "truecolor");

    let child = pair.slave.spawn_command(cmd).map_err(|e| e.to_string())?;

    let writer = pair.master.take_writer().map_err(|e| e.to_string())?;
    let mut reader = pair.master.try_clone_reader().map_err(|e| e.to_string())?;

    let reported_cwd: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));

    let session = PtySession {
        writer: Arc::new(Mutex::new(writer)),
        master: Arc::new(Mutex::new(pair.master)),
        child: Arc::new(Mutex::new(child)),
        reported_cwd: reported_cwd.clone(),
    };

    state
        .sessions
        .lock()
        .map_err(|e| e.to_string())?
        .insert(id.clone(), session);

    let session_id = id.clone();
    let app_handle = app.clone();
    thread::spawn(move || {
        // O shell reporta o diretório atual pelo prompt (OSC 9;9 / OSC 7);
        // o scanner remonta sequências partidas entre leituras.
        let mut osc = crate::osc::OscScanner::new();
        let mut buffer = [0u8; 4096];
        while let Ok(n) = reader.read(&mut buffer) {
            if n == 0 {
                break;
            }
            let chunk = &buffer[..n];

            if let Some(cwd) = osc.push(chunk) {
                if let Ok(mut slot) = reported_cwd.lock() {
                    *slot = Some(cwd);
                }
            }

            let data = String::from_utf8_lossy(chunk).to_string();
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
