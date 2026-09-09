use tauri::AppHandle;

#[tauri::command]
pub fn new_window(app: AppHandle) -> Result<(), String> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let id = format!("win-{}", now);
    tauri::WebviewWindowBuilder::new(&app, id, tauri::WebviewUrl::default())
        .title("xterminium")
        .inner_size(900.0, 620.0)
        .decorations(false)
        .build()
        .map_err(|e| e.to_string())?;
    Ok(())
}
