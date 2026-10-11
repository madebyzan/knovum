use std::{cell::OnceCell, path::Path, sync::mpsc::sync_channel, time::Duration};

use quicklook::{PreviewItem, QuickLookPanel};

// The panel is AppKit state that only its own thread may touch, and `Retained<QLPreviewPanel>` is
// not `Send`, so Tauri cannot manage it either: the main thread keeps the only copy.
thread_local! {
    static PANEL: OnceCell<QuickLookPanel> = const { OnceCell::new() };
}

const DISPATCH_TIMEOUT: Duration = Duration::from_secs(5);
const PANEL_UNAVAILABLE: &str = "Quick Look could not open the system preview panel";
const PATH_UNREADABLE: &str = "Quick Look could not read this path as a file URL";
const NO_ANSWER: &str = "The Quick Look preview panel did not answer in time";

pub(super) fn toggle(app: &tauri::AppHandle, path: &Path) -> Result<(), String> {
    let target = path.to_path_buf();
    let (sender, receiver) = sync_channel(1);
    app.run_on_main_thread(move || {
        let _ = sender.send(preview(&target));
    })
    .map_err(|error| error.to_string())?;
    receiver
        .recv_timeout(DISPATCH_TIMEOUT)
        .map_err(|_| NO_ANSWER.to_owned())?
}

pub(super) fn preview(path: &Path) -> Result<(), String> {
    PANEL.with(|cell| {
        if cell.get().is_none() {
            // `shared()` installs a fresh data source and delegate on the AppKit singleton, so it
            // belongs here, once, on the thread that will own the panel for the rest of the run.
            if let Some(panel) = QuickLookPanel::shared() {
                let _ = cell.set(panel);
            }
        }
        let panel = cell.get().ok_or_else(|| PANEL_UNAVAILABLE.to_owned())?;
        let item =
            PreviewItem::from_file_url(path, None).ok_or_else(|| PATH_UNREADABLE.to_owned())?;
        let url = item.absolute_url_string();
        let already_showing = panel.is_visible()
            && panel
                .current_preview_item()
                .and_then(|current| current.absolute_url_string())
                == url;
        if already_showing {
            panel.hide();
            return Ok(());
        }
        panel.set_items(vec![item]);
        panel.reload_if_dirty();
        panel.show();
        Ok(())
    })
}
