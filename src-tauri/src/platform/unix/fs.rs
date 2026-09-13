//! Diretórios e permissões no POSIX.

use std::path::{Path, PathBuf};

/// Diretório home do usuário.
///
/// `HOME` é o padrão POSIX. Os fallbacks cobrem ambientes onde ele não é
/// exportado (serviços, containers) e shells que herdam variáveis do Windows.
pub fn home_dir() -> PathBuf {
    if let Ok(home) = std::env::var("HOME") {
        if !home.is_empty() {
            return PathBuf::from(home);
        }
    }
    if let Ok(userprofile) = std::env::var("USERPROFILE") {
        if !userprofile.is_empty() {
            return PathBuf::from(userprofile);
        }
    }
    if let (Ok(drive), Ok(path)) = (std::env::var("HOMEDRIVE"), std::env::var("HOMEPATH")) {
        let combined = format!("{drive}{path}");
        if !combined.is_empty() {
            return PathBuf::from(combined);
        }
    }
    PathBuf::from(".")
}

/// Diretório de configuração do app.
///
/// Respeita o XDG Base Directory Specification: `$XDG_CONFIG_HOME/xterminium`
/// ou `~/.config/xterminium`.
pub fn config_dir() -> PathBuf {
    if let Ok(config_home) = std::env::var("XDG_CONFIG_HOME") {
        if !config_home.is_empty() {
            return PathBuf::from(config_home).join("xterminium");
        }
    }
    if let Ok(home) = std::env::var("HOME") {
        if !home.is_empty() {
            return PathBuf::from(home).join(".config").join("xterminium");
        }
    }
    PathBuf::from(".").join(".config").join("xterminium")
}

/// Restringe o arquivo ao dono (0600). Necessário porque o JSON de
/// configuração guarda hosts/portas do SSH.
pub fn restrict_file_permissions(path: &Path) {
    use std::os::unix::fs::PermissionsExt;
    let _ = std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600));
}

/// Normaliza um caminho reportado pelo shell via OSC 7 / OSC 9;9.
///
/// No POSIX o caminho já chega no formato final: não há nada a reescrever.
pub fn normalize_reported_path(path: String) -> String {
    path
}
