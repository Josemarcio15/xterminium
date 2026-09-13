use tauri::AppHandle;

/// Cria uma nova janela do terminal.

#[tauri::command]
pub async fn new_window(app: AppHandle) -> Result<(), String> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let id = format!("win-{}", now);
    let win = tauri::WebviewWindowBuilder::new(&app, id, tauri::WebviewUrl::default())
        .title("xterminium")
        .inner_size(900.0, 620.0)
        .decorations(false)
        .transparent(true)
        .build()
        .map_err(|e| e.to_string())?;

    // Transparência nativa: sem isso a janela nova não fica 100% transparente
    // (implementação por SO em `crate::platform`).
    crate::platform::apply_transparency(&win);

    Ok(())
}
