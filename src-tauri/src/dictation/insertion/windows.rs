use windows::Win32::{
    Foundation::HWND,
    UI::{
        Input::KeyboardAndMouse::{
            SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP, VIRTUAL_KEY,
            VK_CONTROL, VK_V,
        },
        WindowsAndMessaging::{
            GetForegroundWindow, GetWindowTextLengthW, GetWindowTextW, GetWindowThreadProcessId,
        },
    },
};

use super::PasteError;

#[derive(Clone)]
pub(crate) struct InsertionTarget {
    hwnd: isize,
    pid: u32,
    display_name: String,
}

impl InsertionTarget {
    pub(crate) fn display_name(&self) -> &str {
        &self.display_name
    }
}

pub(super) fn capture_target() -> Result<InsertionTarget, String> {
    let hwnd = unsafe { GetForegroundWindow() };
    if hwnd.0.is_null() {
        return Err("No foreground application is available".into());
    }
    let mut pid = 0;
    unsafe { GetWindowThreadProcessId(hwnd, Some(&mut pid)) };
    if pid == 0 {
        return Err("The foreground window has no process identity".into());
    }
    Ok(InsertionTarget {
        hwnd: hwnd.0 as isize,
        pid,
        display_name: window_title(hwnd).unwrap_or_else(|| format!("PID {pid}")),
    })
}

pub(super) fn is_focused(expected: &InsertionTarget) -> bool {
    capture_target()
        .is_ok_and(|current| current.hwnd == expected.hwnd && current.pid == expected.pid)
}

pub(super) const fn has_permission() -> bool {
    true
}

pub(super) const fn permission_message() -> &'static str {
    "Windows denied keyboard input to the target application"
}

pub(super) fn paste() -> Result<(), PasteError> {
    let inputs = [
        key(VK_CONTROL, false),
        key(VK_V, false),
        key(VK_V, true),
        key(VK_CONTROL, true),
    ];
    let sent = unsafe { SendInput(&inputs, std::mem::size_of::<INPUT>() as i32) };
    if sent != inputs.len() as u32 {
        let releases = [key(VK_V, true), key(VK_CONTROL, true)];
        unsafe { SendInput(&releases, std::mem::size_of::<INPUT>() as i32) };
        return Err(PasteError::permission_denied(
            "Windows blocked keyboard input to the foreground application",
        ));
    }
    Ok(())
}

fn key(value: VIRTUAL_KEY, released: bool) -> INPUT {
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: value,
                dwFlags: if released {
                    KEYEVENTF_KEYUP
                } else {
                    Default::default()
                },
                ..Default::default()
            },
        },
    }
}

fn window_title(hwnd: HWND) -> Option<String> {
    let length = unsafe { GetWindowTextLengthW(hwnd) };
    if length <= 0 {
        return None;
    }
    let mut buffer = vec![0_u16; length as usize + 1];
    let read = unsafe { GetWindowTextW(hwnd, &mut buffer) };
    (read > 0).then(|| String::from_utf16_lossy(&buffer[..read as usize]))
}
