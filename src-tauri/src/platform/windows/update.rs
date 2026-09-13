//! Auto-update no Windows: executa o instalador oficial (`install.ps1`).

use std::process::Command;

/// O instalador do Windows não pede senha: quem controla a elevação é o UAC,
/// tratado pelo próprio Windows durante a execução.
pub fn update_needs_password() -> bool {
    false
}

/// Baixa e executa o instalador via PowerShell.
pub fn run_update_installer(_password: Option<String>) -> Result<(), String> {
    let script =
        "irm https://raw.githubusercontent.com/Josemarcio15/xterminium/main/install.ps1 | iex";
    let status = Command::new("powershell.exe")
        .args([
            "-NoProfile",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            script,
        ])
        .status()
        .map_err(|e| format!("Erro ao executar script no PowerShell: {}", e))?;

    if !status.success() {
        return Err(format!(
            "O instalador do PowerShell finalizou com código: {:?}",
            status.code()
        ));
    }
    Ok(())
}
