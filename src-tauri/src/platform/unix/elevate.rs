//! Elevação de privilégio local no POSIX (`sudo`).

use tokio::io::AsyncWriteExt;

/// Utilitários que podem ser elevados no gerenciador de arquivos local.
///
/// Lista estrita: evita que a senha do usuário seja usada para rodar um binário
/// arbitrário que veio da interface.
const ALLOWED_BINS: [&str; 4] = ["touch", "mkdir", "mv", "rm"];

/// Executa um comando local com privilégios de administrador.
///
/// A senha vai pelo `stdin` do `sudo -S` (nunca por argumento, que ficaria
/// visível na lista de processos).
pub async fn exec_local_sudo(password: &str, command: &str) -> Result<(), String> {
    let trimmed = command.trim();
    if trimmed.is_empty() {
        return Err("Comando vazio fornecido para sudo local.".to_string());
    }

    // Separa os argumentos de forma segura respeitando strings entre aspas se houver
    let raw_args = match shlex::split(trimmed) {
        Some(args) if !args.is_empty() => args,
        _ => return Err("Comando inválido ou malformado para sudo local.".to_string()),
    };

    let bin = &raw_args[0];
    if !ALLOWED_BINS.contains(&bin.as_str()) {
        return Err(format!(
            "Execução do binário '{}' não é permitida via sudo.",
            bin
        ));
    }

    let mut child = tokio::process::Command::new("sudo")
        .arg("-S")
        .arg("-p")
        .arg("")
        .arg(bin)
        .args(&raw_args[1..])
        .stdin(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .stdout(std::process::Stdio::null())
        .spawn()
        .map_err(|e| format!("Erro ao iniciar processo sudo local: {}", e))?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(format!("{}\n", password).as_bytes())
            .await
            .map_err(|e| format!("Erro ao enviar senha para sudo local: {}", e))?;
    }

    let output = child
        .wait_with_output()
        .await
        .map_err(|e| format!("Erro ao aguardar processo sudo local: {}", e))?;

    if !output.status.success() {
        let err_msg = String::from_utf8_lossy(&output.stderr);
        let cleaned_err = err_msg.trim();
        let lower = cleaned_err.to_lowercase();
        if lower.contains("incorrect password")
            || lower.contains("try again")
            || lower.contains("authentication failed")
            || lower.contains("authentication failure")
        {
            return Err("Senha sudo incorreta.".to_string());
        }
        return Err(format!("Falha no sudo local: {}", cleaned_err));
    }

    Ok(())
}
