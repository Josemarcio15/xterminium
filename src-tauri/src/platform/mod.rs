//! Código específico de sistema operacional.
//!
//! Este é o **único** lugar do backend com `#[cfg]` de dispatch: o resto do app
//! chama `crate::platform::*` sem saber em que SO está rodando. Cada SO
//! implementa a mesma assinatura pública e o que é compartilhado fica fora
//! daqui.
//!
//! - `windows/` — Win32/ConPTY (PEB para cwd, `set_background_color`).
//! - `unix/` — POSIX (`/proc`, GTK). O que for exclusivo do Linux fica com
//!   `#[cfg(target_os = "linux")]` dentro do próprio módulo.
//!
//! Cada lado tem os mesmos módulos: `fs`, `shells`, `update`, `elevate`,
//! `process` e `transparency`.
//!
//! Contrato espelhado entre os dois lados (mesmos nomes e assinaturas):
//! `home_dir`, `config_dir`, `restrict_file_permissions`,
//! `normalize_reported_path`, `process_cwd`, `pty_foreground_process`,
//! `apply_transparency`, `detect_shells`, `default_shell_path`,
//! `run_update_installer`, `update_needs_password`, `exec_local_sudo` + tipo
//! tipo `ForegroundProcess`.

#[cfg(unix)]
mod unix;
#[cfg(windows)]
mod windows;

#[cfg(unix)]
pub use unix::*;
#[cfg(windows)]
pub use windows::*;

/// Processo em foreground dentro do PTY (o filho que está de fato rodando).
///
/// Contrato compartilhado: cada SO preenche o que consegue. Quando `name` vem
/// vazio, o chamador não deve sobrescrever o nome do processo anterior.
#[derive(Clone, Default)]
pub struct ForegroundProcess {
    pub name: String,
    pub cmdline: Option<String>,
    pub is_ssh: bool,
}
