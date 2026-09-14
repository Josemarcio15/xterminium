use crate::platform::CpuTempData;
use std::ffi::c_void;

const FILE_MAP_READ: u32 = 4;

extern "system" {
    fn OpenFileMappingW(
        dwDesiredAccess: u32,
        bInheritHandle: i32,
        lpName: *const u16,
    ) -> *mut c_void;
    fn MapViewOfFile(
        hFileMappingObject: *mut c_void,
        dwDesiredAccess: u32,
        dwFileOffsetHigh: u32,
        dwFileOffsetLow: u32,
        dwNumberOfBytesToMap: usize,
    ) -> *mut c_void;
    fn UnmapViewOfFile(lpBaseAddress: *const c_void) -> i32;
    fn CloseHandle(hObject: *mut c_void) -> i32;
}

#[repr(C, packed(4))]
struct CoreTempSharedDataEx {
    ui_load: [u32; 256],
    ui_tj_max: [u32; 128],
    ui_core_cnt: u32,
    ui_cpu_cnt: u32,
    f_temp: [f32; 256],
    f_vid: f32,
    f_cpu_speed: f32,
    f_fsb_speed: f32,
    f_multiplier: f32,
    s_cpu_name: [u8; 100],
    uc_fahrenheit: u8,
    uc_delta_to_tj_max: u8,
}

/// Lê temperaturas de cada núcleo via Shared Memory do Core Temp no Windows.
pub fn read_cpu_temp() -> Option<CpuTempData> {
    let names: [&[u16]; 2] = [
        &[
            b'C' as u16, b'o' as u16, b'r' as u16, b'e' as u16, b'T' as u16, b'e' as u16,
            b'm' as u16, b'p' as u16, b'M' as u16, b'a' as u16, b'p' as u16, b'p' as u16,
            b'i' as u16, b'n' as u16, b'g' as u16, b'O' as u16, b'b' as u16, b'j' as u16,
            b'e' as u16, b'c' as u16, b't' as u16, b'E' as u16, b'x' as u16, 0,
        ],
        &[
            b'C' as u16, b'o' as u16, b'r' as u16, b'e' as u16, b'T' as u16, b'e' as u16,
            b'm' as u16, b'p' as u16, b'M' as u16, b'a' as u16, b'p' as u16, b'p' as u16,
            b'i' as u16, b'n' as u16, b'g' as u16, b'O' as u16, b'b' as u16, b'j' as u16,
            b'e' as u16, b'c' as u16, b't' as u16, 0,
        ],
    ];

    for name in names {
        unsafe {
            let handle = OpenFileMappingW(FILE_MAP_READ, 0, name.as_ptr());
            if handle.is_null() {
                continue;
            }

            let view = MapViewOfFile(
                handle,
                FILE_MAP_READ,
                0,
                0,
                std::mem::size_of::<CoreTempSharedDataEx>(),
            );
            if view.is_null() {
                CloseHandle(handle);
                continue;
            }

            let data = &*(view as *const CoreTempSharedDataEx);
            let core_count = (data.ui_core_cnt as usize).min(256);
            if core_count == 0 {
                UnmapViewOfFile(view);
                CloseHandle(handle);
                continue;
            }

            let mut temps = Vec::with_capacity(core_count);
            let mut max_t: f32 = 0.0;
            let mut sum_t: f32 = 0.0;

            for i in 0..core_count {
                let t = data.f_temp[i];
                temps.push(t);
                if t > max_t {
                    max_t = t;
                }
                sum_t += t;
            }

            let avg_t = if core_count > 0 {
                sum_t / (core_count as f32)
            } else {
                0.0
            };

            UnmapViewOfFile(view);
            CloseHandle(handle);

            return Some(CpuTempData {
                tdie: max_t,
                max_temp: max_t,
                avg_temp: avg_t,
                core_temps: temps,
            });
        }
    }

    None
}
