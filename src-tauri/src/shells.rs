//! Detecção dos shells instalados no sistema.
//!
//! Windows: PowerShell 7 (pwsh), Windows PowerShell, CMD, WSL, Git Bash / MSYS2, Nushell.
//! Linux/macOS: shells de `/etc/shells` (zsh, bash, fish, sh, ...) + pwsh/nu do PATH.
//!
//! A lista é usada para popular o seletor de shell da interface. O shell escolhido
//! é enviado ao `spawn_pty` através dos parâmetros `command`/`args`.

use std::path::Path;

/// Máximo de shells retornados, para evitar listas gigantes em ambientes incomuns.
const MAX_SHELLS: usize = 40;

#[derive(Clone, serde::Serialize)]
pub struct ShellInfo {
    pub id: String,
    pub name: String,
    pub path: String,
    pub args: Vec<String>,
    pub kind: String,
    pub recommended: bool,
}

/// Cria um identificador estável a partir do caminho do shell.
fn slugify(path: &str) -> String {
    let mut out = String::new();
    let mut pending_dash = false;
    for ch in path.chars() {
        if ch.is_ascii_alphanumeric() {
            if pending_dash && !out.is_empty() {
                out.push('-');
            }
            pending_dash = false;
            out.push(ch.to_ascii_lowercase());
        } else {
            pending_dash = true;
        }
    }
    while out.ends_with('-') {
        out.pop();
    }
    out
}

fn is_file(path: &str) -> bool {
    !path.is_empty() && Path::new(path).is_file()
}

fn normalize(path: &str) -> String {
    path.trim()
        .trim_end_matches(['/', '\\'])
        .to_ascii_lowercase()
}

/// Procura um executável no PATH (respeitando PATHEXT no Windows).
fn which(cmd: &str) -> Option<String> {
    let path_var = std::env::var_os("PATH")?;

    #[cfg(target_os = "windows")]
    let exts: Vec<String> = std::env::var("PATHEXT")
        .unwrap_or_else(|_| ".EXE;.CMD;.BAT;.COM".to_string())
        .split(';')
        .map(|e| e.trim().to_ascii_lowercase())
        .filter(|e| !e.is_empty())
        .collect();
    #[cfg(not(target_os = "windows"))]
    let exts: Vec<String> = Vec::new();

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

/// Classifica o shell a partir do nome do executável.
fn kind_from_path(path: &str) -> &'static str {
    let file = path
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or(path)
        .to_ascii_lowercase();
    let stem = file.strip_suffix(".exe").unwrap_or(&file);

    match stem {
        "pwsh" => "pwsh",
        "powershell" => "powershell",
        "cmd" => "cmd",
        "wsl" => "wsl",
        "zsh" => "zsh",
        "fish" => "fish",
        "nu" => "nu",
        "sh" => "sh",
        "bash" => {
            if path.to_ascii_lowercase().contains("git") {
                "git-bash"
            } else {
                "bash"
            }
        }
        _ => "other",
    }
}

fn display_name(path: &str, kind: &str) -> String {
    match kind {
        "pwsh" => "PowerShell 7 (pwsh)",
        "powershell" => "Windows PowerShell",
        "cmd" => "Prompt de Comando (cmd)",
        "wsl" => "WSL - Windows Subsystem for Linux",
        "git-bash" => "Git Bash",
        "bash" => "Bash",
        "zsh" => "Zsh",
        "fish" => "Fish",
        "nu" => "Nushell (nu)",
        "sh" => "Bourne Shell (sh)",
        _ => path.rsplit(['/', '\\']).next().unwrap_or(path),
    }
    .to_string()
}

fn push_unique(list: &mut Vec<ShellInfo>, path: String, args: Vec<String>, recommended: &str) {
    if list.len() >= MAX_SHELLS || !is_file(&path) {
        return;
    }
    let key = normalize(&path);
    if list.iter().any(|s| normalize(&s.path) == key) {
        return;
    }
    let kind = kind_from_path(&path).to_string();
    let name = display_name(&path, &kind);
    let is_recommended = normalize(&path) == normalize(recommended);
    list.push(ShellInfo {
        id: slugify(&path),
        name,
        path,
        args,
        kind,
        recommended: is_recommended,
    });
}

#[cfg(target_os = "windows")]
fn detect_shells(recommended: &str) -> Vec<ShellInfo> {
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

#[cfg(not(target_os = "windows"))]
fn detect_shells(recommended: &str) -> Vec<ShellInfo> {
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
    #[cfg(target_os = "windows")]
    {
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
    #[cfg(not(target_os = "windows"))]
    {
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
}

/// Lista os shells disponíveis no sistema para o seletor de configurações.
#[tauri::command]
pub fn list_shells() -> Vec<ShellInfo> {
    let recommended = default_shell_path();
    detect_shells(&recommended)
}
