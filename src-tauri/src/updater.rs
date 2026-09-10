use std::process::Command;

/// Dispara o script de atualização do xterminium de acordo com a plataforma (Linux / Windows)
#[tauri::command]
pub async fn run_update_installer(password: Option<String>) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        #[cfg(target_os = "linux")]
        {
            let mut cmd = Command::new("bash");
            // Se senha foi fornecida, exporta SUDO_PASSWORD e configura elevate inline
            // caso o script do GitHub ainda não tenha sido atualizado na branch main
            let script = r#"
                if [ -n "$SUDO_PASSWORD" ]; then
                    sudo() {
                        printf '%s\n' "$SUDO_PASSWORD" | /usr/bin/sudo -S -p '' "$@"
                    }
                    export -f sudo
                fi
                curl -fsSL https://raw.githubusercontent.com/Josemarcio15/xterminium/main/install.sh | bash
            "#;
            cmd.arg("-c").arg(script);

            if let Some(pwd) = password {
                cmd.env("SUDO_PASSWORD", pwd);
            }

            let output = cmd
                .output()
                .map_err(|e| format!("Erro ao executar script de atualização: {}", e))?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                let stdout = String::from_utf8_lossy(&output.stdout);
                let err_msg = if !stderr.trim().is_empty() {
                    stderr.trim().to_string()
                } else if !stdout.trim().is_empty() {
                    stdout.lines().last().unwrap_or("Falha na execução").to_string()
                } else {
                    format!("Código de erro: {:?}", output.status.code())
                };

                // Trata erro de senha incorreta do sudo
                if err_msg.to_lowercase().contains("incorrect password")
                    || err_msg.to_lowercase().contains("senha incorreta")
                    || err_msg.to_lowercase().contains("authentication failure")
                {
                    return Err("Senha de administrador incorreta. Tente novamente.".to_string());
                }

                return Err(format!("Falha na instalação: {}", err_msg));
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
