use tauri::{DragDropEvent, Runtime, Webview, WebviewEvent};
use tauri_plugin_fs::FsExt;

/// Mirrors a user-confirmed webview drag/drop into the filesystem plugin scope.
/// Dialog selections already receive this grant from the dialog plugin; webview
/// webview drops need the same grant before custom Rust commands may read the path.
pub fn authorize_drop<R: Runtime>(webview: &Webview<R>, event: &WebviewEvent) {
    let WebviewEvent::DragDrop(DragDropEvent::Drop { paths, .. }) = event else {
        return;
    };
    let scope = webview.fs_scope();
    for path in paths {
        let granted = if path.is_dir() {
            scope.allow_directory(path, true)
        } else {
            scope.allow_file(path)
        };
        if let Err(error) = granted {
            log::warn!(
                "failed to authorize dropped path {}: {error}",
                path.display()
            );
        }
    }
}
