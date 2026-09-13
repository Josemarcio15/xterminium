use tauri::WebviewWindow;

/// Torna a janela transparente de verdade no GTK.
///
/// São três passos: marcar a janela como "app paintable", usar um visual RGBA
/// (senão o GTK pinta o fundo preto/cinza) e zerar via CSS o background do
/// container nativo — o CSS do webview sozinho não cobre o container do GTK.
#[cfg(target_os = "linux")]
pub fn apply_transparency(window: &WebviewWindow) {
    use gtk::prelude::*;

    let Ok(gtk_win) = window.gtk_window() else {
        return;
    };

    gtk_win.set_app_paintable(true);
    if let Some(screen) = gtk::prelude::WidgetExt::screen(&gtk_win) {
        if let Some(visual) = screen.rgba_visual() {
            gtk_win.set_visual(Some(&visual));
        }

        let css_provider = gtk::CssProvider::new();
        let css = "
            window, .background {
                background-color: transparent !important;
                border: none !important;
                box-shadow: none !important;
            }
        ";
        if css_provider.load_from_data(css.as_bytes()).is_ok() {
            gtk::StyleContext::add_provider_for_screen(
                &screen,
                &css_provider,
                gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );
        }
    }
}

/// Fora do Linux não há GTK; hoje o lado unix do app é só Linux.
#[cfg(not(target_os = "linux"))]
pub fn apply_transparency(_window: &WebviewWindow) {}
