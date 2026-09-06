use super::types::FileEntry;
use std::path::{Path, PathBuf};
use tokio::fs as local_fs;

/// Utilitário para listar diretórios locais
pub async fn list_local_directory(dir_path: &Path) -> Result<Vec<FileEntry>, String> {
    let mut read_dir = local_fs::read_dir(dir_path)
        .await
        .map_err(|e| format!("Erro ao ler pasta local: {}", e))?;

    let mut entries = Vec::new();

    while let Ok(Some(entry)) = read_dir.next_entry().await {
        let metadata = match entry.metadata().await {
            Ok(m) => m,
            Err(_) => continue,
        };

        let is_dir = metadata.is_dir();
        let is_symlink = metadata.is_symlink();
        let file_name = entry.file_name().to_string_lossy().to_string();
        let full_path = entry.path().to_string_lossy().to_string();

        let modified = metadata
            .modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs());

        entries.push(FileEntry {
            name: file_name,
            path: full_path,
            is_dir,
            is_symlink,
            size: metadata.len(),
            modified,
        });
    }

    entries.sort_by(|a, b| match (a.is_dir, b.is_dir) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
    });

    Ok(entries)
}

/// Determina a pasta home local de forma segura
pub fn get_local_home_dir() -> PathBuf {
    if let Ok(home) = std::env::var("HOME") {
        PathBuf::from(home)
    } else {
        PathBuf::from(".")
    }
}

/// Cria um novo diretório na máquina local
pub async fn create_local_directory(path: &str) -> Result<(), String> {
    local_fs::create_dir_all(path)
        .await
        .map_err(|e| format!("Erro ao criar diretório local: {}", e))
}

/// Cria um novo arquivo vazio na máquina local
pub async fn create_local_file_entry(path: &str) -> Result<(), String> {
    local_fs::File::create(path)
        .await
        .map_err(|e| format!("Erro ao criar arquivo local: {}", e))?;
    Ok(())
}

/// Renomeia ou move um arquivo/pasta na máquina local
pub async fn rename_local_entry(old_path: &str, new_path: &str) -> Result<(), String> {
    local_fs::rename(old_path, new_path)
        .await
        .map_err(|e| format!("Erro ao renomear item local: {}", e))
}

/// Remove um arquivo na máquina local
pub async fn remove_local_file_entry(path: &str) -> Result<(), String> {
    local_fs::remove_file(path)
        .await
        .map_err(|e| format!("Erro ao remover arquivo local: {}", e))
}

/// Remove um diretório na máquina local
pub async fn remove_local_directory(path: &str) -> Result<(), String> {
    local_fs::remove_dir_all(path)
        .await
        .map_err(|e| format!("Erro ao remover pasta local: {}", e))
}

