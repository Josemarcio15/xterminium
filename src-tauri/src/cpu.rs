pub use crate::platform::CpuTempData;
use std::sync::Mutex;
use sysinfo::{CpuRefreshKind, RefreshKind, System};
use tauri::State;

pub struct CpuMonitorState(pub Mutex<System>);

impl Default for CpuMonitorState {
    fn default() -> Self {
        let mut sys = System::new_with_specifics(
            RefreshKind::nothing().with_cpu(CpuRefreshKind::everything()),
        );
        sys.refresh_cpu_usage();
        Self(Mutex::new(sys))
    }
}

/// Retorna a porcentagem de uso global da CPU (0.0 a 100.0).
#[tauri::command]
pub fn get_cpu_usage(state: State<'_, CpuMonitorState>) -> f32 {
    let mut sys = match state.0.lock() {
        Ok(g) => g,
        Err(poisoned) => poisoned.into_inner(),
    };
    sys.refresh_cpu_usage();
    sys.global_cpu_usage()
}

/// Lê as temperaturas da CPU via implementação da plataforma
/// (Core Temp no Windows, hwmon/thermal no Linux).
#[tauri::command]
pub fn get_cpu_temp() -> Option<CpuTempData> {
    crate::platform::read_cpu_temp()
}
