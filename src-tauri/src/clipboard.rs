use tauri::AppHandle;
use tauri_plugin_clipboard_manager::ClipboardExt;

#[tauri::command]
pub fn read_clipboard(app: AppHandle) -> Result<String, String> {
    // No Linux (especialmente Wayland), o wl-paste é a fonte mais confiável para comunicação entre apps externos
    #[cfg(target_os = "linux")]
    {
        // 1. Tenta ler a área de transferência normal do Wayland (onde vai o Ctrl+C do navegador/outros apps)
        if let Ok(output) = std::process::Command::new("wl-paste")
            .args(["--no-newline"])
            .output()
        {
            if output.status.success() {
                if let Ok(text) = String::from_utf8(output.stdout) {
                    if !text.is_empty() {
                        return Ok(text);
                    }
                }
            }
        }

        // 2. Tenta ler o buffer primário do Wayland (texto selecionado com mouse no Linux)
        if let Ok(output) = std::process::Command::new("wl-paste")
            .args(["--primary", "--no-newline"])
            .output()
        {
            if output.status.success() {
                if let Ok(text) = String::from_utf8(output.stdout) {
                    if !text.is_empty() {
                        return Ok(text);
                    }
                }
            }
        }

        // 3. Fallback para X11 / XWayland via xclip
        if let Ok(output) = std::process::Command::new("xclip")
            .args(["-selection", "clipboard", "-o"])
            .output()
        {
            if output.status.success() {
                if let Ok(text) = String::from_utf8(output.stdout) {
                    if !text.is_empty() {
                        return Ok(text);
                    }
                }
            }
        }
    }

    // Fallback geral via API oficial do Tauri
    if let Ok(text) = app.clipboard().read_text() {
        if !text.is_empty() {
            return Ok(text);
        }
    }

    Ok(String::new())
}

#[tauri::command]
pub fn write_clipboard(app: AppHandle, text: String) -> Result<(), String> {
    // 1. Tenta a API oficial do Tauri
    let primary_res = app.clipboard().write_text(text.clone());

    // 2. No Linux, se falhar ou se estivermos em Wayland, assegura via ferramentas do sistema
    #[cfg(target_os = "linux")]
    {
        if primary_res.is_err() {
            // Tenta wl-copy para Wayland
            if let Ok(mut child) = std::process::Command::new("wl-copy")
                .stdin(std::process::Stdio::piped())
                .spawn()
            {
                use std::io::Write;
                if let Some(mut stdin) = child.stdin.take() {
                    let _ = stdin.write_all(text.as_bytes());
                }
                let _ = child.wait();
                return Ok(());
            }

            // Tenta xclip para X11
            if let Ok(mut child) = std::process::Command::new("xclip")
                .args(["-selection", "clipboard"])
                .stdin(std::process::Stdio::piped())
                .spawn()
            {
                use std::io::Write;
                if let Some(mut stdin) = child.stdin.take() {
                    let _ = stdin.write_all(text.as_bytes());
                }
                let _ = child.wait();
                return Ok(());
            }
        }
    }

    primary_res.map_err(|e| e.to_string())
}
