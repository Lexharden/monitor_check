use chrono::prelude::*;
use sysinfo::{System};
use windows::Win32::Foundation::{HWND, LPARAM, BOOL};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetWindowTextLengthW, GetWindowTextW, IsWindowVisible,
};
use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use serde::{Serialize};

#[derive(Debug, Serialize)]
pub struct ApplicationInfo {
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct MonitorData {
    #[serde(with = "my_date_format")]
    pub boot_time: DateTime<Local>,
    pub applications: Vec<ApplicationInfo>,
    pub cpu_usage: f32,
    pub memory_usage: u64, // Mantener u64 aquí
}

mod my_date_format {
    use chrono::{DateTime, Local, SecondsFormat};
    use serde::{self, Serializer, Deserializer, Deserialize};

    pub fn serialize<S>(date: &DateTime<Local>, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
    {
        let s = date.to_rfc3339_opts(SecondsFormat::Secs, true);
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Local>, D::Error>
        where
            D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(DateTime::parse_from_rfc3339(&s).unwrap().with_timezone(&Local))
    }
}

pub fn get_monitor_data(system: &mut System, start_time: &DateTime<Local>) -> MonitorData {
    system.refresh_all();
    let uptime = sysinfo::System::uptime();
    let boot_time = *start_time - chrono::Duration::seconds(uptime as i64);

    let mut applications = Vec::new();
    unsafe {
        let _ = EnumWindows(Some(enum_windows_proc), LPARAM(&mut applications as *mut _ as isize));
    }

    let cpu_usage = system.global_cpu_info().cpu_usage();
    let memory_usage = system.used_memory(); // Mantener en KB

    MonitorData {
        boot_time,
        applications,
        cpu_usage,
        memory_usage,
    }
}

extern "system" fn enum_windows_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let applications: &mut Vec<ApplicationInfo> = unsafe { &mut *(lparam.0 as *mut Vec<ApplicationInfo>) };
    unsafe {
        if IsWindowVisible(hwnd).as_bool() {
            let length = GetWindowTextLengthW(hwnd);
            if length > 0 {
                let mut buffer: Vec<u16> = vec![0; (length + 1) as usize];
                let read_length = GetWindowTextW(hwnd, &mut buffer);
                if read_length > 0 {
                    let window_name = OsString::from_wide(&buffer[..read_length as usize]).to_string_lossy().into_owned();
                    applications.push(ApplicationInfo { name: window_name });
                }
            }
        }
    }
    BOOL(1)
}
