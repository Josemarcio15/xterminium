//! Diretórios e permissões no Windows.

use std::path::{Path, PathBuf};

/// Diretório home do usuário.
///
/// `USERPROFILE` primeiro: é o caminho nativo do Windows. `HOME` vem de
/// toolchains MSYS/Cygwin (Git Bash, WSL) como caminho estilo POSIX
/// (`/c/Users/...`), que não serve para `CreateProcess` — por isso só é usado
/// como último recurso.
pub fn home_dir() -> PathBuf {
    if let Ok(profile) = std::env::var("USERPROFILE") {
        if !profile.is_empty() {
            return PathBuf::from(profile);
        }
    }
    if let (Ok(drive), Ok(path)) = (std::env::var("HOMEDRIVE"), std::env::var("HOMEPATH")) {
        let combined = format!("{drive}{path}");
        if !combined.is_empty() {
            return PathBuf::from(combined);
        }
    }
    if let Ok(home) = std::env::var("HOME") {
        if !home.is_empty() {
            return PathBuf::from(home);
        }
    }
    PathBuf::from(".")
}

/// Diretório de configuração do app (`%APPDATA%\xterminium`).
pub fn config_dir() -> PathBuf {
    if let Ok(appdata) = std::env::var("APPDATA") {
        if !appdata.is_empty() {
            return PathBuf::from(appdata).join("xterminium");
        }
    }
    if let Ok(profile) = std::env::var("USERPROFILE") {
        if !profile.is_empty() {
            return PathBuf::from(profile).join(".config").join("xterminium");
        }
    }
    PathBuf::from(".").join(".config").join("xterminium")
}

/// No Windows não existe `chmod`: o ACL herdado do `%APPDATA%` já restringe o
/// arquivo ao próprio usuário, então não há nada a fazer.
pub fn restrict_file_permissions(_path: &Path) {}

/// Normaliza um caminho reportado pelo shell via OSC 7 / OSC 9;9.
///
/// O OSC 7 usa o formato de URL `file://host/C:/Users/...`: a barra inicial é
/// sintaxe da URL, não do caminho, então é removida antes do drive letter
/// (sem isso um `IdentityFile`/cwd viraria `/C:/...`).
pub fn normalize_reported_path(path: String) -> String {
    match path.strip_prefix('/') {
        Some(rest) if is_drive_path(rest) => rest.to_string(),
        _ => path,
    }
}

/// `C:/...` -> o caminho começa com um drive do Windows.
fn is_drive_path(path: &str) -> bool {
    let bytes = path.as_bytes();
    bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':'
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn remove_a_barra_da_url_de_drive() {
        assert_eq!(
            normalize_reported_path("/C:/Users/Marcio".to_string()),
            "C:/Users/Marcio"
        );
    }

    #[test]
    fn preserva_caminho_posix() {
        assert_eq!(
            normalize_reported_path("/home/marcio".to_string()),
            "/home/marcio"
        );
    }
}
