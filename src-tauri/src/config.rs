use std::path::PathBuf;
use std::sync::Mutex;
use serde::{Deserialize, Serialize};

static CUSTOM_CONFIG_DIR: Mutex<Option<PathBuf>> = Mutex::new(None);

fn get_pointer_file() -> PathBuf {
    crate::platform::config_dir().join(".config_location")
}

pub fn get_config_dir() -> PathBuf {
    if let Ok(guard) = CUSTOM_CONFIG_DIR.lock() {
        if let Some(ref path) = *guard {
            return path.clone();
        }
    }

    let ptr = get_pointer_file();
    if ptr.exists() {
        if let Ok(custom) = std::fs::read_to_string(ptr) {
            let custom_trimmed = custom.trim();
            if !custom_trimmed.is_empty() {
                let p = PathBuf::from(custom_trimmed);
                if let Ok(mut guard) = CUSTOM_CONFIG_DIR.lock() {
                    *guard = Some(p.clone());
                }
                return p;
            }
        }
    }

    crate::platform::config_dir()
}

pub fn get_default_config_dir() -> PathBuf {
    crate::platform::config_dir()
}

/// Resolve um caminho de arquivo de forma universal e tolerante a Windows/Linux.
/// - Converte `~` ou `$HOME` para o home do SO atual
/// - Se for um caminho absoluto de outro SO (ex: `C:\Users\Marcio\.ssh\id_ed25519` no Linux ou `/home/marcio/...` no Windows),
///   tenta adaptar relativamente ao HOME ou encontrar o arquivo com o mesmo nome em `~/.ssh/`.
pub fn resolve_universal_path(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    let home = crate::platform::home_dir();

    // 1. Tratamento para `~` ou `$HOME`
    if trimmed == "~" || trimmed == "$HOME" {
        return home.to_string_lossy().to_string();
    }
    if trimmed.starts_with("~/") || trimmed.starts_with("~\\") {
        let sub = &trimmed[2..].replace('\\', "/");
        return home.join(sub).to_string_lossy().to_string();
    }
    if trimmed.starts_with("$HOME/") || trimmed.starts_with("$HOME\\") {
        let sub = &trimmed[6..].replace('\\', "/");
        return home.join(sub).to_string_lossy().to_string();
    }

    // 2. Se o arquivo existe exatamente como especificado
    let p = std::path::Path::new(trimmed);
    if p.exists() {
        return trimmed.to_string();
    }

    // 3. Fallback inteligente para chaves SSH cruzadas de outro SO
    // Ex: "C:\Users\...\.ssh\id_ed25519" no Linux ou "/home/.../.ssh/id_ed25519" no Windows
    let normalized = trimmed.replace('\\', "/");
    if let Some(ssh_idx) = normalized.find("/.ssh/") {
        let sub = &normalized[ssh_idx + 6..]; // nome do arquivo dentro de .ssh
        let candidate = home.join(".ssh").join(sub);
        if candidate.exists() {
            return candidate.to_string_lossy().to_string();
        }
    }

    // Se tiver apenas o nome de arquivo em ~/.ssh
    let file_name = trimmed.rsplit(['/', '\\']).next().unwrap_or(trimmed);
    let candidate = home.join(".ssh").join(file_name);
    if candidate.exists() {
        return candidate.to_string_lossy().to_string();
    }

    trimmed.to_string()
}

#[tauri::command]
pub fn resolve_path(path: String) -> String {
    resolve_universal_path(&path)
}

#[derive(Serialize, Deserialize)]
pub struct ConfigLocationInfo {
    pub current_path: String,
    pub default_path: String,
    pub is_custom: bool,
}

#[derive(Serialize, Deserialize)]
pub struct MigrationCheckResult {
    pub target_exists: bool,
    pub target_has_files: bool,
    pub conflicting_files: Vec<String>,
}

#[tauri::command]
pub fn get_config_location() -> Result<ConfigLocationInfo, String> {
    let current = get_config_dir();
    let default = get_default_config_dir();
    let is_custom = current != default;
    Ok(ConfigLocationInfo {
        current_path: current.to_string_lossy().to_string(),
        default_path: default.to_string_lossy().to_string(),
        is_custom,
    })
}

#[tauri::command]
pub fn check_migration_target(target_path: String) -> Result<MigrationCheckResult, String> {
    let target = PathBuf::from(target_path.trim());
    if !target.exists() {
        return Ok(MigrationCheckResult {
            target_exists: false,
            target_has_files: false,
            conflicting_files: vec![],
        });
    }

    let current = get_config_dir();
    let mut conflicts = Vec::new();
    let mut has_files = false;

    if let Ok(entries) = std::fs::read_dir(&target) {
        for entry in entries.flatten() {
            if let Ok(ft) = entry.file_type() {
                if ft.is_file() {
                    has_files = true;
                    let fname = entry.file_name().to_string_lossy().to_string();
                    if fname.ends_with(".json") && current.join(&fname).exists() {
                        conflicts.push(fname);
                    }
                }
            }
        }
    }

    Ok(MigrationCheckResult {
        target_exists: true,
        target_has_files: has_files,
        conflicting_files: conflicts,
    })
}

#[tauri::command]
pub fn change_config_location(target_path: String, overwrite: bool) -> Result<String, String> {
    let trimmed = target_path.trim();
    if trimmed.is_empty() {
        return Err("O caminho não pode ser vazio".to_string());
    }

    let target = PathBuf::from(trimmed);
    let current = get_config_dir();

    if current == target {
        return Ok(target.to_string_lossy().to_string());
    }

    std::fs::create_dir_all(&target).map_err(|e| format!("Falha ao criar diretório de destino: {}", e))?;

    // Mover / copiar arquivos da pasta antiga para a nova pasta
    if current.exists() {
        if let Ok(entries) = std::fs::read_dir(&current) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    let fname = entry.file_name();
                    let fname_str = fname.to_string_lossy();
                    // Evitar mover o arquivo ponteiro se ele estiver na raiz
                    if fname_str == ".config_location" {
                        continue;
                    }
                    if fname_str.ends_with(".json") {
                        let dest_file = target.join(&fname);
                        if dest_file.exists() && !overwrite {
                            // Se já existir e não quer sobrescrever, mantém o do destino e remove o antigo ou preserva
                            continue;
                        }
                        if let Err(e) = std::fs::copy(&path, &dest_file) {
                            return Err(format!("Falha ao mover arquivo {}: {}", fname_str, e));
                        }
                        let _ = std::fs::remove_file(&path);
                    }
                }
            }
        }
    }

    // Atualizar ponteiro
    let default = get_default_config_dir();
    let ptr = get_pointer_file();

    if target == default {
        // Se voltou para o padrão, podemos deletar o ponteiro
        if ptr.exists() {
            let _ = std::fs::remove_file(&ptr);
        }
        if let Ok(mut guard) = CUSTOM_CONFIG_DIR.lock() {
            *guard = None;
        }
    } else {
        if let Some(parent) = ptr.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        std::fs::write(&ptr, target.to_string_lossy().to_string())
            .map_err(|e| format!("Falha ao salvar ponteiro de configuração: {}", e))?;
        if let Ok(mut guard) = CUSTOM_CONFIG_DIR.lock() {
            *guard = Some(target.clone());
        }
    }

    Ok(target.to_string_lossy().to_string())
}

#[tauri::command]
pub fn pick_config_folder() -> Result<Option<String>, String> {
    let current = get_config_dir();
    let folder = rfd::FileDialog::new()
        .set_directory(current)
        .pick_folder();

    Ok(folder.map(|p| p.to_string_lossy().to_string()))
}

#[tauri::command]
pub fn load_config(filename: String) -> Result<String, String> {
    let dir = get_config_dir();
    let file_path = dir.join(format!("{}.json", filename));
    if file_path.exists() {
        let content = std::fs::read_to_string(file_path).map_err(|e| e.to_string())?;
        if filename == "ssh" && !content.trim().is_empty() {
            let _ = sync_ssh_config(&content);
        }
        Ok(content)
    } else {
        Ok("".to_string())
    }
}

#[tauri::command]
pub fn save_config(filename: String, content: String) -> Result<(), String> {
    let dir = get_config_dir();
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let file_path = dir.join(format!("{}.json", filename));
    std::fs::write(file_path, &content).map_err(|e| e.to_string())?;

    if filename == "ssh" {
        if let Err(e) = sync_ssh_config(&content) {
            log::warn!("Failed to sync ~/.ssh/config: {}", e);
        }
    }

    Ok(())
}

#[derive(serde::Deserialize)]
struct SshHostEntry {
    #[allow(dead_code)]
    id: Option<String>,
    label: Option<String>,
    user: Option<String>,
    ip: Option<String>,
    port: Option<String>,
    key: Option<String>,
}

fn get_ssh_dir() -> Option<PathBuf> {
    let home = crate::platform::home_dir();
    // Sem home resolvido o helper devolve um caminho relativo; nesse caso é
    // melhor não mexer em `~/.ssh` nenhum.
    if home.is_absolute() {
        Some(home.join(".ssh"))
    } else {
        None
    }
}

pub fn sync_ssh_config(json_content: &str) -> Result<(), String> {
    let hosts: Vec<SshHostEntry> = match serde_json::from_str(json_content) {
        Ok(h) => h,
        Err(_) => return Ok(()),
    };

    let ssh_dir = match get_ssh_dir() {
        Some(dir) => dir,
        None => return Ok(()),
    };

    if !ssh_dir.exists() {
        let _ = std::fs::create_dir_all(&ssh_dir);
    }

    let config_path = ssh_dir.join("config");
    let existing_content = if config_path.exists() {
        std::fs::read_to_string(&config_path).unwrap_or_default()
    } else {
        String::new()
    };

    let begin_marker = "# >>> BEGIN XTERMINIUM MANAGED HOSTS >>>";
    let end_marker = "# <<< END XTERMINIUM MANAGED HOSTS <<<";

    let mut generated_block = String::new();
    generated_block.push_str(begin_marker);
    generated_block.push('\n');
    generated_block.push_str(&format!(
        "# This block is automatically managed by Xterminium based on {}\n",
        get_config_dir().join("ssh.json").display()
    ));
    generated_block.push_str("# Any manual changes inside this block may be overwritten.\n\n");

    for host in &hosts {
        let label = match &host.label {
            Some(l) if !l.trim().is_empty() => l.trim().to_string(),
            _ => continue,
        };

        let ip = match &host.ip {
            Some(ip) if !ip.trim().is_empty() => ip.trim(),
            _ => continue,
        };

        // Permite múltiplos aliases se o label tiver caracteres especiais ou espaços
        // Usamos o label limpo (sem espaços para alias SSH)
        let alias = label.replace(' ', "_");

        generated_block.push_str(&format!("Host {}\n", alias));
        generated_block.push_str(&format!("    HostName {}\n", ip));

        if let Some(user) = &host.user {
            if !user.trim().is_empty() {
                generated_block.push_str(&format!("    User {}\n", user.trim()));
            }
        }

        if let Some(port) = &host.port {
            if !port.trim().is_empty() && port.trim() != "22" {
                generated_block.push_str(&format!("    Port {}\n", port.trim()));
            }
        }

        if let Some(key) = &host.key {
            let trimmed_key = key.trim();
            if !trimmed_key.is_empty() {
                let expanded_key = resolve_universal_path(trimmed_key);
                generated_block.push_str(&format!("    IdentityFile {}\n", expanded_key));
            }
        }
        generated_block.push('\n');
    }

    generated_block.push_str(end_marker);

    let new_content = if let (Some(start), Some(end)) = (
        existing_content.find(begin_marker),
        existing_content.find(end_marker),
    ) {
        if start <= end {
            let prefix = &existing_content[..start];
            let suffix_index = end + end_marker.len();
            let suffix = &existing_content[suffix_index..];
            format!("{}{}{}", prefix, generated_block, suffix)
        } else {
            format!("{}\n\n{}\n", existing_content.trim_end(), generated_block)
        }
    } else {
        if existing_content.trim().is_empty() {
            format!("{}\n", generated_block)
        } else {
            format!("{}\n\n{}\n", existing_content.trim_end(), generated_block)
        }
    };

    std::fs::write(&config_path, new_content).map_err(|e| e.to_string())?;

    // O JSON guarda hosts/portas do SSH: restringe ao dono (0600 no POSIX,
    // no-op no Windows). Detalhe por SO em `crate::platform`.
    crate::platform::restrict_file_permissions(&config_path);

    Ok(())
}
