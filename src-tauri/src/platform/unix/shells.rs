//! Detecção dos shells instalados no POSIX.

use crate::shells::{is_file, push_unique, ShellInfo};

/// Procura um executável no PATH (no POSIX o nome é literal, sem `PATHEXT`).
fn which(cmd: &str) -> Option<String> {
    let path_var = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path_var) {
        let direct = dir.join(cmd);
        if direct.is_file() {
            return Some(direct.to_string_lossy().into_owned());
        }
    }
    None
}

/// Shells encontrados: `$SHELL`, `/etc/shells`, caminhos comuns (cobre distros
/// sem `/etc/shells`, macOS e Homebrew) e opcionais instalados via gerenciador
/// de pacotes.
pub fn detect_shells(recommended: &str) -> Vec<ShellInfo> {
    let mut list = Vec::new();

    // Shell atual do usuário
    if let Ok(user_shell) = std::env::var("SHELL") {
        if !user_shell.trim().is_empty() {
            push_unique(&mut list, user_shell, vec![], recommended);
        }
    }

    // Shells registrados pelo sistema
    if let Ok(content) = std::fs::read_to_string("/etc/shells") {
        for line in content.lines() {
            let entry = line.trim();
            if entry.is_empty() || entry.starts_with('#') {
                continue;
            }
            push_unique(&mut list, entry.to_string(), vec![], recommended);
        }
    }

    // Caminhos comuns (cobre distros sem /etc/shells e macOS/Homebrew)
    for candidate in [
        "/bin/bash",
        "/usr/bin/bash",
        "/bin/zsh",
        "/usr/bin/zsh",
        "/usr/local/bin/zsh",
        "/opt/homebrew/bin/zsh",
        "/bin/fish",
        "/usr/bin/fish",
        "/bin/sh",
        "/usr/bin/sh",
    ] {
        push_unique(&mut list, candidate.to_string(), vec![], recommended);
    }

    // Shells opcionais instalados via gerenciadores de pacote
    for cmd in ["pwsh", "nu"] {
        if let Some(p) = which(cmd) {
            push_unique(&mut list, p, vec![], recommended);
        }
    }

    list
}

/// Shell usado quando nenhuma preferência foi definida pelo usuário.
pub fn default_shell_path() -> String {
    // 1. Respeita $SHELL do usuário
    if let Ok(user_shell) = std::env::var("SHELL") {
        if !user_shell.is_empty() && is_file(&user_shell) {
            return user_shell;
        }
    }
    // 2. Zsh
    for candidate in ["/bin/zsh", "/usr/bin/zsh", "/usr/local/bin/zsh"] {
        if is_file(candidate) {
            return candidate.to_string();
        }
    }
    // 3. Bash
    for candidate in ["/bin/bash", "/usr/bin/bash"] {
        if is_file(candidate) {
            return candidate.to_string();
        }
    }
    // 4. POSIX sh
    "/bin/sh".to_string()
}
