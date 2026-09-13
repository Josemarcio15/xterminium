//! Auto-update no POSIX: executa o instalador oficial (`install.sh`).

#[cfg(target_os = "linux")]
use std::process::Command;

/// O pacote `.deb` é instalado com `sudo`, então o app precisa pedir a senha
/// antes de disparar o instalador.
pub fn update_needs_password() -> bool {
    true
}

/// Baixa e executa o instalador via `bash`.
///
/// A senha do administrador é passada por variável de ambiente
/// (`SUDO_PASSWORD`), nunca pela linha de comando — assim não aparece na lista
/// de processos.
#[cfg(target_os = "linux")]
pub fn run_update_installer(password: Option<String>) -> Result<(), String> {
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
            stdout
                .lines()
                .last()
                .unwrap_or("Falha na execução")
                .to_string()
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

/// Fora do Linux o app não publica instalador (o lado unix é só Linux).
#[cfg(not(target_os = "linux"))]
pub fn run_update_installer(_password: Option<String>) -> Result<(), String> {
    Err("Plataforma não suportada para auto-update.".to_string())
}
