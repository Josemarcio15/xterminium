//! Detecção dos shells instalados no Windows.

use crate::shells::{is_file, push_unique, ShellInfo};

/// Procura um executável no PATH, respeitando `PATHEXT` (`pwsh` -> `pwsh.exe`).
fn which(cmd: &str) -> Option<String> {
    let path_var = std::env::var_os("PATH")?;
    let exts: Vec<String> = std::env::var("PATHEXT")
        .unwrap_or_else(|_| ".EXE;.CMD;.BAT;.COM".to_string())
        .split(';')
        .map(|e| e.trim().to_ascii_lowercase())
        .filter(|e| !e.is_empty())
        .collect();

    for dir in std::env::split_paths(&path_var) {
        let direct = dir.join(cmd);
        if direct.is_file() {
            return Some(direct.to_string_lossy().into_owned());
        }
        for ext in &exts {
            let with_ext = dir.join(format!("{}{}", cmd, ext));
            if with_ext.is_file() {
                return Some(with_ext.to_string_lossy().into_owned());
            }
        }
    }
    None
}

/// Shells encontrados: PowerShell 7, Windows PowerShell, CMD, WSL, Git Bash /
/// MSYS2 e Nushell.
pub fn detect_shells(recommended: &str) -> Vec<ShellInfo> {
    let mut list = Vec::new();

    // PowerShell 7+ (instalação via winget/MSI/store)
    if let Some(p) = which("pwsh") {
        push_unique(&mut list, p, vec![], recommended);
    }

    // Shells nativos do Windows
    if let Ok(root) = std::env::var("SystemRoot") {
        push_unique(
            &mut list,
            format!(
                "{}\\System32\\WindowsPowerShell\\v1.0\\powershell.exe",
                root
            ),
            vec![],
            recommended,
        );
        push_unique(
            &mut list,
            format!("{}\\System32\\cmd.exe", root),
            vec![],
            recommended,
        );
    }
    if let Some(p) = which("powershell") {
        push_unique(&mut list, p, vec![], recommended);
    }
    if let Some(p) = which("cmd") {
        push_unique(&mut list, p, vec![], recommended);
    }

    // WSL (abre a distro padrão)
    if let Some(p) = which("wsl") {
        push_unique(&mut list, p, vec![], recommended);
    }

    // Git Bash / MSYS2 — usa shell de login para carregar /etc/profile
    let login_args = vec!["--login".to_string(), "-i".to_string()];
    let mut git_candidates: Vec<String> = Vec::new();
    if let Ok(pf) = std::env::var("ProgramFiles") {
        git_candidates.push(format!("{}\\Git\\bin\\bash.exe", pf));
    }
    if let Ok(pf) = std::env::var("ProgramFiles(x86)") {
        git_candidates.push(format!("{}\\Git\\bin\\bash.exe", pf));
    }
    if let Ok(local) = std::env::var("LOCALAPPDATA") {
        git_candidates.push(format!("{}\\Programs\\Git\\bin\\bash.exe", local));
    }
    for candidate in git_candidates {
        push_unique(&mut list, candidate, login_args.clone(), recommended);
    }
    if let Some(p) = which("bash") {
        push_unique(&mut list, p, login_args, recommended);
    }

    // Nushell
    if let Some(p) = which("nu") {
        push_unique(&mut list, p, vec![], recommended);
    }

    list
}

/// Shell usado quando nenhuma preferência foi definida pelo usuário.
pub fn default_shell_path() -> String {
    if let Ok(system_root) = std::env::var("SystemRoot") {
        let ps_path = format!(
            "{}\\System32\\WindowsPowerShell\\v1.0\\powershell.exe",
            system_root
        );
        if is_file(&ps_path) {
            return ps_path;
        }
    }
    if let Some(p) = which("powershell") {
        return p;
    }
    "powershell.exe".to_string()
}
