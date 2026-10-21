pub(crate) mod artifacts;
mod catalog;
mod connection;
pub mod conversation;
mod direct_endpoints;
pub mod discovery;
mod error;
mod help;
mod installed_clients;
pub(crate) mod permissions;
pub mod prompt;
mod protocol;
pub mod registry;
mod resource_links;
pub mod session;
pub(crate) mod task;
mod updates;

pub use discovery::detect_external_agent_installations;

use tauri::{AppHandle, Manager};

#[derive(Default)]
struct ExternalAgentInitialization(tokio::sync::OnceCell<Result<(), String>>);

pub(crate) fn install(app: &AppHandle) {
    app.manage(ExternalAgentInitialization::default());
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        if let Err(error) = ready(&app).await {
            log::error!("External Agent runtime initialization failed: {error}");
        }
    });
}

pub(crate) async fn ready(app: &AppHandle) -> Result<(), String> {
    let state = app.state::<ExternalAgentInitialization>();
    state
        .0
        .get_or_init(|| async {
            crate::backup::ready(app).await?;
            task::discard_interrupted(app).map_err(|error| error.to_string())
        })
        .await
        .clone()
}
