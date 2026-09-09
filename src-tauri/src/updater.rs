use std::process::Command;

/// Dispara o script de atualização do xterminium de acordo com a plataforma (Linux / Windows)
#[tauri::command]
pub async fn run_update_installer() -> Result<(), String> {
    tokio::task::spawn_blocking(|| {
        #[cfg(target_os = "linux")]
        {
            let script = "curl -fsSL https://raw.githubusercontent.com/Josemarcio15/xterminium/main/install.sh | bash";
            let status = Command::new("bash")
                .arg("-c")
                .arg(script)
                .status()
                .map_err(|e| format!("Erro ao executar script de atualização: {}", e))?;

            if !status.success() {
                return Err(format!("O instalador finalizou com código de erro: {:?}", status.code()));
            }
            Ok(())
        }

        #[cfg(target_os = "windows")]
        {
            let script = "irm https://raw.githubusercontent.com/Josemarcio15/xterminium/main/install.ps1 | iex";
            let status = Command::new("powershell.exe")
                .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", script])
                .status()
                .map_err(|e| format!("Erro ao executar script no PowerShell: {}", e))?;

            if !status.success() {
                return Err(format!("O instalador do PowerShell finalizou com código: {:?}", status.code()));
            }
            Ok(())
        }

        #[cfg(not(any(target_os = "linux", target_os = "windows")))]
        {
            Err("Plataforma não suportada para auto-update.".to_string())
        }
    })
    .await
    .map_err(|e| format!("Falha na task de atualização: {}", e))?
}

/// Retorna a versão atual da aplicação definida no Cargo.toml
#[tauri::command]
pub fn get_app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
