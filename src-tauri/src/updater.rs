/// Dispara o script de atualização do xterminium de acordo com a plataforma
/// (Linux / Windows). O que cada SO executa vive em `crate::platform`.
#[tauri::command]
pub async fn run_update_installer(_password: Option<String>) -> Result<(), String> {
    tokio::task::spawn_blocking(move || crate::platform::run_update_installer(_password))
        .await
        .map_err(|e| format!("Falha na task de atualização: {}", e))?
}

/// Retorna a versão atual da aplicação definida no Cargo.toml
#[tauri::command]
pub fn get_app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Informa se o fluxo de atualização deste SO exige senha de administrador.
///
/// A interface usa isso para decidir se abre o prompt de `sudo` — a política
/// específica de cada SO vive em `crate::platform`.
#[tauri::command]
pub fn update_needs_password() -> bool {
    crate::platform::update_needs_password()
}
