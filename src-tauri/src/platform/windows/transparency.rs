use tauri::WebviewWindow;

/// Torna a janela transparente de verdade.
///
/// No Windows basta pintar o container nativo com alpha 0; sem isso o webview
/// fica com fundo branco/cinza mesmo com o CSS transparente.
///
/// Best-effort: falhas são ignoradas (comportamento tolerante a drivers/versões
/// que não aceitam a cor transparente).
pub fn apply_transparency(window: &WebviewWindow) {
    let _ = window.set_background_color(Some(tauri::webview::Color(0, 0, 0, 0)));
    let _ = window.set_shadow(false);
}
