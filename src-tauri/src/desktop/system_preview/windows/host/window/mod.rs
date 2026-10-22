use std::{
    ffi::c_void,
    path::Path,
    ptr,
    sync::atomic::{AtomicIsize, Ordering},
};

use windows::{
    core::{w, Error, HSTRING},
    Win32::{
        Foundation::{HINSTANCE, HWND, LPARAM, WPARAM},
        System::LibraryLoader::GetModuleHandleW,
        UI::{
            Shell::IPreviewHandler,
            WindowsAndMessaging::{
                CreateWindowExW, DestroyWindow, DispatchMessageW, GetMessageW, PostMessageW,
                SetForegroundWindow, ShowWindow, TranslateMessage, CW_USEDEFAULT, MSG, SW_SHOW,
                WINDOW_EX_STYLE, WM_CLOSE, WS_CLIPCHILDREN, WS_OVERLAPPEDWINDOW,
            },
        },
    },
};

mod state;

use state::HostState;

static ACTIVE_WINDOW: AtomicIsize = AtomicIsize::new(0);

pub(super) struct PreviewWindow {
    hwnd: HWND,
    state: Box<HostState>,
}

impl PreviewWindow {
    pub(super) fn create(
        handler: IPreviewHandler,
        source: &Path,
        owner: isize,
    ) -> Result<Self, String> {
        state::register_class()?;
        let mut state = Box::new(HostState::new(handler));
        let title = preview_title(source);
        let hwnd = unsafe {
            CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                w!("KnovumSystemPreview"),
                &title,
                WS_OVERLAPPEDWINDOW | WS_CLIPCHILDREN,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                960,
                720,
                Some(HWND(owner as *mut c_void)),
                None,
                Some(module_instance()?),
                Some(ptr::from_mut(&mut *state).cast_const().cast()),
            )
        }
        .map_err(|error| format!("Windows could not create the preview host window: {error}"))?;
        if let Err(error) = state.start(hwnd) {
            let _ = unsafe { DestroyWindow(hwnd) };
            return Err(error);
        }
        Ok(Self { hwnd, state })
    }

    pub(super) fn show(&self) {
        let current = self.hwnd.0 as isize;
        let previous = ACTIVE_WINDOW.swap(current, Ordering::AcqRel);
        if previous != 0 && previous != current {
            let _ = unsafe {
                PostMessageW(
                    Some(HWND(previous as *mut c_void)),
                    WM_CLOSE,
                    WPARAM(0),
                    LPARAM(0),
                )
            };
        }
        let _ = unsafe { ShowWindow(self.hwnd, SW_SHOW) };
        let _ = unsafe { SetForegroundWindow(self.hwnd) };
    }

    pub(super) fn run(&mut self) -> Result<(), String> {
        let mut message = MSG::default();
        loop {
            match unsafe { GetMessageW(&mut message, None, 0, 0) }.0 {
                -1 => return Err(Error::from_thread().to_string()),
                0 => return Ok(()),
                _ => {
                    let _ = unsafe { TranslateMessage(&message) };
                    unsafe { DispatchMessageW(&message) };
                }
            }
        }
    }
}

impl Drop for PreviewWindow {
    fn drop(&mut self) {
        self.state.unload();
        let _ = unsafe { DestroyWindow(self.hwnd) };
        let _ = ACTIVE_WINDOW.compare_exchange(
            self.hwnd.0 as isize,
            0,
            Ordering::AcqRel,
            Ordering::Acquire,
        );
    }
}

pub(super) fn module_instance() -> Result<HINSTANCE, String> {
    unsafe { GetModuleHandleW(None) }
        .map(HINSTANCE::from)
        .map_err(|error| error.to_string())
}

fn preview_title(source: &Path) -> HSTRING {
    let name = source.file_name().unwrap_or(source.as_os_str());
    HSTRING::from(format!("{} — Knovum", name.to_string_lossy()))
}
