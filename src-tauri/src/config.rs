use std::path::PathBuf;

pub fn get_config_dir() -> PathBuf {
    if let Ok(appdata) = std::env::var("APPDATA") {
        PathBuf::from(appdata).join("xterminium")
    } else if let Ok(config_home) = std::env::var("XDG_CONFIG_HOME") {
        PathBuf::from(config_home).join("xterminium")
    } else if let Ok(home) = std::env::var("HOME") {
        PathBuf::from(home).join(".config").join("xterminium")
    } else if let Ok(userprofile) = std::env::var("USERPROFILE") {
        PathBuf::from(userprofile).join(".config").join("xterminium")
    } else {
        PathBuf::from(".").join(".config").join("xterminium")
    }
}

#[tauri::command]
pub fn load_config(filename: String) -> Result<String, String> {
    let dir = get_config_dir();
    let file_path = dir.join(format!("{}.json", filename));
    if file_path.exists() {
        std::fs::read_to_string(file_path).map_err(|e| e.to_string())
    } else {
        Ok("".to_string())
    }
}

#[tauri::command]
pub fn save_config(filename: String, content: String) -> Result<(), String> {
    let dir = get_config_dir();
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let file_path = dir.join(format!("{}.json", filename));
    std::fs::write(file_path, content).map_err(|e| e.to_string())?;
    Ok(())
}
