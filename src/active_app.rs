use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowTextLengthW, GetWindowTextW};
use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;

pub fn get_active_window() -> Option<String> {
    unsafe {
        let hwnd: HWND = GetForegroundWindow();
        if hwnd.0 != 0 {
            let length = GetWindowTextLengthW(hwnd);
            if length > 0 {
                let mut buffer: Vec<u16> = vec![0; (length + 1) as usize];
                let read_length = GetWindowTextW(hwnd, &mut buffer);
                if read_length > 0 {
                    return Some(OsString::from_wide(&buffer[..read_length as usize]).to_string_lossy().into_owned());
                }
            }
        }
    }
    None
}
