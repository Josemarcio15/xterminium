use std::path::PathBuf;

pub fn get_config_dir() -> PathBuf {
    if let Ok(appdata) = std::env::var("APPDATA") {
        PathBuf::from(appdata).join("xterminium")
    } else if let Ok(config_home) = std::env::var("XDG_CONFIG_HOME") {
        PathBuf::from(config_home).join("xterminium")
    } else if let Ok(home) = std::env::var("HOME") {
        PathBuf::from(home).join(".config").join("xterminium")
    } else if let Ok(userprofile) = std::env::var("USERPROFILE") {
        PathBuf::from(userprofile)
            .join(".config")
            .join("xterminium")
    } else {
        PathBuf::from(".").join(".config").join("xterminium")
    }
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
    if let Ok(home) = std::env::var("HOME") {
        Some(PathBuf::from(home).join(".ssh"))
    } else if let Ok(userprofile) = std::env::var("USERPROFILE") {
        Some(PathBuf::from(userprofile).join(".ssh"))
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
    generated_block.push_str("# This block is automatically managed by Xterminium based on ~/.config/xterminium/ssh.json\n");
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
                let expanded_key = if trimmed_key.starts_with("~/") {
                    if let Ok(home) = std::env::var("HOME") {
                        trimmed_key.replacen("~", &home, 1)
                    } else if let Ok(userprofile) = std::env::var("USERPROFILE") {
                        trimmed_key.replacen("~", &userprofile, 1)
                    } else {
                        trimmed_key.to_string()
                    }
                } else {
                    trimmed_key.to_string()
                };
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

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&config_path, std::fs::Permissions::from_mode(0o600));
    }

    Ok(())
}
