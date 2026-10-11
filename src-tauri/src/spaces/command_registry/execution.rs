use tauri::ipc::Invoke;

pub(super) fn handler() -> impl Fn(Invoke<tauri::Wry>) -> bool + Send + Sync + 'static {
    knovum_execution_commands!(command_handler)
}
