use crate::platform::ForegroundProcess;

/// Diretório de trabalho atual do shell que roda no PTY.
#[cfg(target_os = "linux")]
pub fn process_cwd(pid: u32) -> Option<String> {
    let target = std::fs::read_link(format!("/proc/{}/cwd", pid)).ok()?;
    Some(target.to_string_lossy().to_string())
}

/// Fora do Linux não há `/proc`; hoje o lado unix do app é só Linux.
#[cfg(not(target_os = "linux"))]
pub fn process_cwd(_pid: u32) -> Option<String> {
    None
}

/// Processo em foreground dentro do PTY: o último filho em execução.
#[cfg(target_os = "linux")]
pub fn pty_foreground_process(pid: u32) -> Option<ForegroundProcess> {
    let leaf_pid = leaf_child_pid(pid)?;
    let mut info = ForegroundProcess::default();

    if let Ok(comm) = std::fs::read_to_string(format!("/proc/{}/comm", leaf_pid)) {
        let name = comm.trim().to_string();
        if name == "ssh" {
            info.is_ssh = true;
        }
        info.name = name;
    }

    if let Ok(raw_cmd) = std::fs::read(format!("/proc/{}/cmdline", leaf_pid)) {
        let parsed = String::from_utf8_lossy(&raw_cmd)
            .replace('\0', " ")
            .trim()
            .to_string();
        if !parsed.is_empty() {
            if parsed.starts_with("ssh ") || parsed == "ssh" {
                info.is_ssh = true;
            }
            info.cmdline = Some(parsed);
        }
    }

    Some(info)
}

/// Fora do Linux não há `/proc`; hoje o lado unix do app é só Linux.
#[cfg(not(target_os = "linux"))]
pub fn pty_foreground_process(_pid: u32) -> Option<ForegroundProcess> {
    None
}

/// Último processo filho em execução no PTY (o "foreground child").
#[cfg(target_os = "linux")]
fn leaf_child_pid(pid: u32) -> Option<u32> {
    let mut leaf: Option<u32> = None;
    let entries = std::fs::read_dir(format!("/proc/{}/task", pid)).ok()?;
    for entry in entries.flatten() {
        let children_path = entry.path().join("children");
        if let Ok(content) = std::fs::read_to_string(children_path) {
            let last = content
                .split_whitespace()
                .filter_map(|p| p.parse::<u32>().ok())
                .last();
            if last.is_some() {
                leaf = last;
            }
        }
    }
    leaf
}
