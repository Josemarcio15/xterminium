use tauri::AppHandle;

#[tauri::command]
pub fn new_window(app: AppHandle) -> Result<(), String> {
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

    #[cfg(target_os = "windows")]
    {
        let _ = win.set_background_color(Some(tauri::webview::Color(0, 0, 0, 0)));
    }

    #[cfg(target_os = "linux")]
    {
        use gtk::prelude::*;
        if let Ok(gtk_win) = win.gtk_window() {
            gtk_win.set_app_paintable(true);
            if let Some(screen) = gtk::prelude::WidgetExt::screen(&gtk_win) {
                if let Some(visual) = screen.rgba_visual() {
                    gtk_win.set_visual(Some(&visual));
                }
            }
        }
    }

    Ok(())
}
