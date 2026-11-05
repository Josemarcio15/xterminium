use crate::platform::CpuTempData;
use std::fs;
use std::path::Path;

/// Lê temperaturas de cada núcleo / CPU no Linux via `/sys/class/hwmon` e `/sys/class/thermal`.
pub fn read_cpu_temp() -> Option<CpuTempData> {
    // 1. Tenta via hwmon (drivers típicos: coretemp para Intel, k10temp/zenpower para AMD, cpu_thermal, etc.)
    if let Some(data) = read_from_hwmon() {
        return Some(data);
    }

    // 2. Fallback via /sys/class/thermal
    read_from_thermal()
}

fn is_cpu_hwmon(name: &str) -> bool {
    let n = name.to_lowercase();
    n.contains("coretemp")
        || n.contains("k10temp")
        || n.contains("zenpower")
        || n.contains("cpu")
        || n.contains("soc_thermal")
}

fn read_from_hwmon() -> Option<CpuTempData> {
    let hwmon_dir = Path::new("/sys/class/hwmon");
    let entries = fs::read_dir(hwmon_dir).ok()?;

    let mut candidate_hwmon: Option<std::path::PathBuf> = None;
    let mut fallback_hwmon: Option<std::path::PathBuf> = None;

    for entry in entries.flatten() {
        let path = entry.path();
        let name_path = path.join("name");
        if let Ok(name_str) = fs::read_to_string(&name_path) {
            let name = name_str.trim();
            if is_cpu_hwmon(name) {
                candidate_hwmon = Some(path);
                break;
            }
        }
        // Fallback: se tiver algum hwmon com temp1_input
        if fallback_hwmon.is_none() && path.join("temp1_input").exists() {
            fallback_hwmon = Some(path);
        }
    }

    let target_dir = candidate_hwmon.or(fallback_hwmon)?;

    let mut core_temps: Vec<f32> = Vec::new();
    let mut tdie: Option<f32> = None;

    // Vasculha temp*_input (temp1_input até temp32_input)
    for idx in 1..=64 {
        let input_file = target_dir.join(format!("temp{}_input", idx));
        if !input_file.exists() {
            continue;
        }

        if let Ok(val_str) = fs::read_to_string(&input_file) {
            if let Ok(milli) = val_str.trim().parse::<f32>() {
                let temp_c = milli / 1000.0;
                if temp_c > 0.0 && temp_c < 140.0 {
                    let label_file = target_dir.join(format!("temp{}_label", idx));
                    let label = fs::read_to_string(&label_file)
                        .unwrap_or_default()
                        .trim()
                        .to_lowercase();

                    if label.contains("tdie") || label.contains("tctl") {
                        tdie = Some(temp_c);
                    }

                    // Se for Core ou se não tiver outras labels, adiciona
                    if label.contains("core") || label.is_empty() || label.contains("tctl") || label.contains("tdie") {
                        core_temps.push(temp_c);
                    }
                }
            }
        }
    }

    if core_temps.is_empty() {
        return None;
    }

    let mut max_temp = core_temps[0];
    let mut sum = 0.0f32;
    for &t in &core_temps {
        if t > max_temp {
            max_temp = t;
        }
        sum += t;
    }
    let avg_temp = sum / (core_temps.len() as f32);

    Some(CpuTempData {
        tdie: tdie.unwrap_or(max_temp),
        max_temp,
        avg_temp,
        core_temps,
    })
}

fn read_from_thermal() -> Option<CpuTempData> {
    let thermal_dir = Path::new("/sys/class/thermal");
    let entries = fs::read_dir(thermal_dir).ok()?;

    let mut temps: Vec<f32> = Vec::new();

    for entry in entries.flatten() {
        let path = entry.path();
        let fname = path.file_name()?.to_string_lossy();
        if !fname.starts_with("thermal_zone") {
            continue;
        }

        let type_path = path.join("type");
        let is_cpu = if let Ok(t) = fs::read_to_string(&type_path) {
            let t_low = t.to_lowercase();
            t_low.contains("cpu") || t_low.contains("x86_pkg_temp") || t_low.contains("acpitz")
        } else {
            false
        };

        let temp_path = path.join("temp");
        if let Ok(val_str) = fs::read_to_string(&temp_path) {
            if let Ok(milli) = val_str.trim().parse::<f32>() {
                let temp_c = if milli > 1000.0 { milli / 1000.0 } else { milli };
                if temp_c > 0.0 && temp_c < 140.0 {
                    if is_cpu {
                        temps.insert(0, temp_c);
                    } else {
                        temps.push(temp_c);
                    }
                }
            }
        }
    }

    if temps.is_empty() {
        return None;
    }

    let max_temp = temps.iter().copied().fold(temps[0], f32::max);
    let avg_temp = temps.iter().sum::<f32>() / (temps.len() as f32);

    Some(CpuTempData {
        tdie: max_temp,
        max_temp,
        avg_temp,
        core_temps: temps,
    })
}
