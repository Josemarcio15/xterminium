//! Metadados de shells e helpers compartilhados entre os sistemas.
//!
//! A **detecção** do que está instalado é específica de cada SO e vive em
//! `crate::platform` (`detect_shells` / `default_shell_path`). Aqui ficam só o
//! DTO que a interface consome e a classificação de caminhos — que é a mesma em
//! qualquer sistema, já que um Linux pode ter `pwsh` no PATH e um Windows pode
//! ter `bash` (Git Bash).
//!
//! A lista é usada para popular o seletor de shell da interface. O shell
//! escolhido é enviado ao `spawn_pty` através dos parâmetros `command`/`args`.
//!
//! A busca no PATH (`which`) também é específica de cada SO e vive em
//! `crate::platform`, porque o Windows resolve por `PATHEXT`.

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

pub fn is_file(path: &str) -> bool {
    !path.is_empty() && Path::new(path).is_file()
}

pub fn normalize(path: &str) -> String {
    path.trim()
        .trim_end_matches(['/', '\\'])
        .to_ascii_lowercase()
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

/// `true` para PowerShell (pwsh ou Windows PowerShell).
///
/// Importa porque o PowerShell nunca chama `chdir` no `Set-Location`: o cwd do
/// processo fica congelado no diretório de lançamento, então o shell precisa
/// reportar o diretório atual pelo prompt (OSC 9;9). Ver `pty.rs`.
pub fn is_powershell(path: &str) -> bool {
    matches!(kind_from_path(path), "pwsh" | "powershell")
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

/// Adiciona à lista se o caminho for um arquivo existente e ainda não listado.
pub fn push_unique(list: &mut Vec<ShellInfo>, path: String, args: Vec<String>, recommended: &str) {
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

/// Lista os shells disponíveis no sistema para o seletor de configurações.
///
/// A varredura em si é feita pelo módulo do SO ativo (`crate::platform`).
#[tauri::command]
pub fn list_shells() -> Vec<ShellInfo> {
    crate::platform::detect_shells(&crate::platform::default_shell_path())
}
