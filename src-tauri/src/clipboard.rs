#[cfg(target_os = "linux")]
use std::io::Write;
use tauri::AppHandle;

#[tauri::command]
pub fn read_clipboard(_app: AppHandle) -> Result<String, String> {
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
        if let Ok(output) = std::process::Command::new("xclip")
            .args(["-selection", "clipboard", "-o"])
            .output()
        {
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
pub fn write_clipboard(_text: String) -> Result<(), String> {
    #[cfg(target_os = "linux")]
    {
        use std::process::Stdio;
        if let Ok(mut child) = std::process::Command::new("wl-copy")
            .stdin(Stdio::piped())
            .spawn()
        {
            if let Some(mut stdin) = child.stdin.take() {
                let _ = stdin.write_all(_text.as_bytes());
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
                let _ = stdin.write_all(_text.as_bytes());
            }
            let _ = child.wait();
            return Ok(());
        }
        return Ok(());
    }
    #[allow(unreachable_code)]
    Err("Native clipboard writing not supported on this platform, use browser fallback".to_string())
}
