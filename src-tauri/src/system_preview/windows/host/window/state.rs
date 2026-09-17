use std::sync::OnceLock;

use windows::{
    core::{w, Error},
    Win32::{
        Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM},
        Graphics::Gdi::{GetSysColorBrush, COLOR_WINDOW},
        UI::{
            Shell::IPreviewHandler,
            WindowsAndMessaging::{
                DefWindowProcW, DestroyWindow, GetClientRect, GetWindowLongPtrW, LoadCursorW,
                PostQuitMessage, RegisterClassW, SetWindowLongPtrW, CREATESTRUCTW, GWLP_USERDATA,
                IDC_ARROW, WM_CLOSE, WM_DESTROY, WM_NCCREATE, WM_SETFOCUS, WM_SIZE, WNDCLASSW,
            },
        },
    },
};

use super::module_instance;

static CLASS: OnceLock<Result<(), String>> = OnceLock::new();

pub(super) struct HostState {
    handler: Option<IPreviewHandler>,
}

impl HostState {
    pub(super) fn new(handler: IPreviewHandler) -> Self {
        Self {
            handler: Some(handler),
        }
    }

    pub(super) fn start(&mut self, hwnd: HWND) -> Result<(), String> {
        let mut rect = RECT::default();
        unsafe { GetClientRect(hwnd, &mut rect) }
            .map_err(|error| format!("Windows could not size the preview host: {error}"))?;
        let handler = self.handler.as_ref().expect("preview handler is installed");
        unsafe { handler.SetWindow(hwnd, &rect) }
            .and_then(|()| unsafe { handler.DoPreview() })
            .map_err(|error| format!("The Windows preview handler could not render: {error}"))
    }

    fn resize(&self, hwnd: HWND) {
        let Some(handler) = self.handler.as_ref() else {
            return;
        };
        let mut rect = RECT::default();
        if unsafe { GetClientRect(hwnd, &mut rect) }.is_ok() {
            let _ = unsafe { handler.SetRect(&rect) };
        }
    }

    fn focus(&self) {
        if let Some(handler) = self.handler.as_ref() {
            let _ = unsafe { handler.SetFocus() };
        }
    }

    pub(super) fn unload(&mut self) {
        if let Some(handler) = self.handler.take() {
            let _ = unsafe { handler.Unload() };
        }
    }
}

pub(super) fn register_class() -> Result<(), String> {
    CLASS.get_or_init(register).clone()
}

fn register() -> Result<(), String> {
    let class = WNDCLASSW {
        lpfnWndProc: Some(window_proc),
        hInstance: module_instance()?,
        hCursor: unsafe { LoadCursorW(None, IDC_ARROW) }.map_err(|error| error.to_string())?,
        hbrBackground: unsafe { GetSysColorBrush(COLOR_WINDOW) },
        lpszClassName: w!("KnovumNativePreview"),
        ..Default::default()
    };
    if unsafe { RegisterClassW(&class) } == 0 {
        Err(format!(
            "Windows could not register the preview window: {}",
            Error::from_win32()
        ))
    } else {
        Ok(())
    }
}

unsafe extern "system" fn window_proc(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if message == WM_NCCREATE {
        let create = unsafe { &*(lparam.0 as *const CREATESTRUCTW) };
        unsafe { SetWindowLongPtrW(hwnd, GWLP_USERDATA, create.lpCreateParams as isize) };
    }
    let state = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut HostState };
    match message {
        WM_SIZE if !state.is_null() => unsafe { (*state).resize(hwnd) },
        WM_SETFOCUS if !state.is_null() => unsafe { (*state).focus() },
        WM_CLOSE => {
            let _ = unsafe { DestroyWindow(hwnd) };
            return LRESULT(0);
        }
        WM_DESTROY => {
            if !state.is_null() {
                unsafe { (*state).unload() };
            }
            unsafe { PostQuitMessage(0) };
            return LRESULT(0);
        }
        _ => {}
    }
    unsafe { DefWindowProcW(hwnd, message, wparam, lparam) }
}
