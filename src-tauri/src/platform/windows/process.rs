use crate::platform::ForegroundProcess;
use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind};

/// Diretório de trabalho atual de um processo.
///
/// O Windows não expõe o cwd no filesystem (não existe `/proc/<pid>/cwd`),
/// então o valor é lido do PEB do processo via `sysinfo`. É exatamente o valor
/// que `cd` (cmd) e `Set-Location` (PowerShell) mantêm sincronizado no shell.
pub fn process_cwd(pid: u32) -> Option<String> {
    let pid = Pid::from_u32(pid);
    let mut system = System::new();
    system.refresh_processes_specifics(
        ProcessesToUpdate::Some(&[pid]),
        true,
        ProcessRefreshKind::nothing().with_cwd(UpdateKind::Always),
    );

    let raw = system.process(pid).and_then(|p| p.cwd())?;
    let raw = raw.to_string_lossy().to_string();
    if raw.is_empty() {
        return None;
    }
    Some(normalize_drive_root(&raw))
}

/// Ainda não implementado no Windows: `get_pty_status` devolve `None` e a
/// detecção de SSH segue dependendo do `cmdline`, hoje só lido no Linux.
pub fn pty_foreground_process(_pid: u32) -> Option<ForegroundProcess> {
    None
}

/// Remove separadores finais sem transformar a raiz do drive em `C:`.
fn normalize_drive_root(path: &str) -> String {
    let trimmed = path.trim_end_matches(['\\', '/']);
    if trimmed.is_empty() {
        path.to_string()
    } else if trimmed.len() == 2 && trimmed.ends_with(':') {
        format!("{trimmed}\\")
    } else {
        trimmed.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preserva_raiz_de_drive() {
        assert_eq!(normalize_drive_root("C:\\"), "C:\\");
        assert_eq!(normalize_drive_root("C:\\Users\\"), "C:\\Users");
        assert_eq!(normalize_drive_root("C:\\Users"), "C:\\Users");
    }

    /// Ler o cwd do PEB funciona para processos que de fato chamam `chdir`
    /// (ex.: `cmd.exe`). PowerShell é um caso à parte: `Set-Location` não chama
    /// `chdir`, então o cwd lido fica congelado no diretório de lançamento.
    #[test]
    fn le_cwd_de_processo_que_chdir() {
        let dir = std::env::temp_dir().canonicalize().expect("temp dir");
        let mut child = std::process::Command::new("cmd.exe")
            .args(["/c", "ping", "-n", "4", "127.0.0.1"])
            .current_dir(&dir)
            .stdout(std::process::Stdio::null())
            .spawn()
            .expect("spawn cmd.exe");

        let lido = process_cwd(child.id());
        let _ = child.kill();

        let lido = std::path::Path::new(&lido.expect("cwd deveria ser legível"))
            .canonicalize()
            .ok();
        assert_eq!(lido, Some(dir));
    }
}
