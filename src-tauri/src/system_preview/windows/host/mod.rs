use std::{
    path::Path,
    sync::mpsc::{sync_channel, SyncSender},
    time::Duration,
};

use windows::Win32::System::Com::{CoInitializeEx, CoUninitialize, COINIT_APARTMENTTHREADED};

use super::handler;

mod window;

const OPEN_TIMEOUT: Duration = Duration::from_secs(10);

pub(super) fn open(source: &Path, owner: isize) -> Result<(), String> {
    let source = source.to_path_buf();
    let (sender, receiver) = sync_channel(1);
    std::thread::Builder::new()
        .name("knovum-native-preview".into())
        .spawn(move || preview_thread(source, owner, sender))
        .map_err(|error| format!("Windows could not start the preview thread: {error}"))?;
    receiver
        .recv_timeout(OPEN_TIMEOUT)
        .map_err(|_| "Windows preview handler did not open in time".to_owned())?
}

fn preview_thread(
    source: std::path::PathBuf,
    owner: isize,
    sender: SyncSender<Result<(), String>>,
) {
    let opened = ComApartment::enter().and_then(|apartment| {
        let handler = handler::load(&source)?;
        window::PreviewWindow::create(handler, &source, owner).map(|window| (apartment, window))
    });
    let (_apartment, mut window) = match opened {
        Ok(opened) => opened,
        Err(error) => {
            let _ = sender.send(Err(error));
            return;
        }
    };
    if sender.send(Ok(())).is_err() {
        return;
    }
    window.show();
    if let Err(error) = window.run() {
        log::error!("Windows native preview failed: {error}");
    }
}

struct ComApartment;

impl ComApartment {
    fn enter() -> Result<Self, String> {
        unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) }
            .ok()
            .map_err(|error| format!("Windows preview STA initialization failed: {error}"))?;
        Ok(Self)
    }
}

impl Drop for ComApartment {
    fn drop(&mut self) {
        unsafe { CoUninitialize() };
    }
}
