use crate::platform::CpuTempData;

/// Lê temperaturas de cada núcleo no Linux via `/sys/class/thermal` ou `/sys/class/hwmon`.
pub fn read_cpu_temp() -> Option<CpuTempData> {
    // Implementação para Linux (hwmon/thermal)
    None
}
