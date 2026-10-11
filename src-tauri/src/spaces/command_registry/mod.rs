#[macro_use]
mod declarations;
macro_rules! command_handler {
    ($($path:path),* $(,)?) => { tauri::generate_handler![$($path),*] };
}
mod execution;
mod knowledge;
mod workspace;

use tauri::ipc::Invoke;

pub(super) fn handler() -> impl Fn(Invoke<tauri::Wry>) -> bool + Send + Sync + 'static {
    let workspace = workspace::handler();
    let knowledge = knowledge::handler();
    let execution = execution::handler();
    move |invoke| {
        workspace(clone_invoke(&invoke)) || knowledge(clone_invoke(&invoke)) || execution(invoke)
    }
}

fn clone_invoke(invoke: &Invoke<tauri::Wry>) -> Invoke<tauri::Wry> {
    Invoke {
        message: invoke.message.clone(),
        resolver: invoke.resolver.clone(),
        acl: invoke.acl.clone(),
    }
}
