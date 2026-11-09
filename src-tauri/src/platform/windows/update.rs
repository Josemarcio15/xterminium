//! Auto-update no Windows: executa o instalador oficial (`install.ps1`).

use std::os::windows::process::CommandExt;
use std::process::Command;

const CREATE_NO_WINDOW: u32 = 0x08000000;

/// O instalador do Windows não pede senha: quem controla a elevação é o UAC,
/// tratado pelo próprio Windows durante a execução.
pub fn update_needs_password() -> bool {
    false
}

/// Baixa e inicia o instalador via PowerShell.
pub fn run_update_installer(_password: Option<String>) -> Result<(), String> {
    let script = "$env:XTERMINIUM_AUTO_UPDATE = '1'; $ProgressPreference = 'SilentlyContinue'; irm https://raw.githubusercontent.com/Josemarcio15/xterminium/main/install.ps1 | iex";
    let output = Command::new("powershell.exe")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            script,
        ])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map_err(|e| format!("Erro ao executar script no PowerShell: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let detail = if !stderr.is_empty() {
            stderr
        } else if !stdout.is_empty() {
            stdout.lines().last().unwrap_or(&stdout).to_string()
        } else {
            format!("código {:?}", output.status.code())
        };
        return Err(format!("Falha ao atualizar: {}", detail));
    }
    Ok(())
}
