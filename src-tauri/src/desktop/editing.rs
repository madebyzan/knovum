use tauri::{AppHandle, Runtime};

use crate::foundation::app_error::AppError;

/// The editing actions AppKit can perform on the focused text control through the responder chain.
#[derive(Clone, Copy, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum EditingCommand {
    Undo,
    Redo,
    Cut,
    Copy,
    Paste,
    SelectAll,
}

#[tauri::command]
pub(crate) async fn perform_editing<R: Runtime>(
    app: AppHandle<R>,
    command: EditingCommand,
) -> Result<bool, AppError> {
    #[cfg(target_os = "macos")]
    {
        let (sender, receiver) = tokio::sync::oneshot::channel();
        app.run_on_main_thread(move || {
            let _ = sender.send(perform_macos(command));
        })
        .map_err(|error| {
            AppError::new(
                "editing_dispatch_failed",
                format!("Editing command could not reach the main thread: {error}"),
            )
        })?;
        receiver.await.map_err(|_| {
            AppError::new(
                "editing_dispatch_cancelled",
                "Editing command did not finish on the main thread",
            )
        })?
    }

    #[cfg(not(target_os = "macos"))]
    {
        let _ = (app, command);
        Err(AppError::new(
            "editing_unavailable",
            "Application-menu text editing is unavailable on this platform",
        ))
    }
}

#[cfg(target_os = "macos")]
fn perform_macos(command: EditingCommand) -> Result<bool, AppError> {
    use objc2::{sel, MainThreadMarker};
    use objc2_app_kit::NSApplication;

    let marker = MainThreadMarker::new().ok_or_else(|| {
        AppError::new(
            "editing_main_thread_unavailable",
            "Editing command is not running on the AppKit main thread",
        )
    })?;
    let selector = match command {
        EditingCommand::Undo => sel!(undo:),
        EditingCommand::Redo => sel!(redo:),
        EditingCommand::Cut => sel!(cut:),
        EditingCommand::Copy => sel!(copy:),
        EditingCommand::Paste => sel!(paste:),
        EditingCommand::SelectAll => sel!(selectAll:),
    };
    let appkit = NSApplication::sharedApplication(marker);
    // SAFETY: the selector is chosen only from AppKit's fixed text editing responder actions.
    // Nil target and sender ask NSApplication to resolve the current first responder.
    Ok(unsafe { appkit.sendAction_to_from(selector, None, None) })
}
