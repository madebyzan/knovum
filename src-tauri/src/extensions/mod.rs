pub(crate) mod availability;
mod connectors;
mod content;
pub(crate) mod contracts;
mod database;
pub(crate) mod mcp;
mod plugins;
pub(crate) mod reclamation;
mod skills;
pub(crate) mod snapshots;

use sqlx::SqlitePool;
use std::{collections::HashMap, sync::Arc};
use tauri::{plugin::TauriPlugin, Manager, Wry};
use tokio::sync::{Mutex, OnceCell};

#[derive(Default)]
pub(crate) struct ExtensionState {
    shutdown: tokio_util::sync::CancellationToken,
    database: OnceCell<SqlitePool>,
    content_lock: Mutex<()>,
    sessions: Mutex<mcp::sessions::Sessions>,
    credential_locks: Mutex<HashMap<String, Arc<Mutex<()>>>>,
    account_locks: Mutex<HashMap<String, Arc<Mutex<()>>>>,
    authorizations: Mutex<HashMap<String, tokio_util::sync::CancellationToken>>,
    oauth_clients: Mutex<
        HashMap<
            String,
            rmcp::transport::auth::AuthClient<
                crate::extensions::mcp::http_client::BoundedHttpClient,
            >,
        >,
    >,
}

pub(crate) fn plugin() -> TauriPlugin<Wry> {
    tauri::plugin::Builder::new("extensions")
        .setup(|app, _| {
            app.manage(ExtensionState::default());
            Ok(())
        })
        .on_event(|app, event| {
            if matches!(event, tauri::RunEvent::Exit) {
                tauri::async_runtime::block_on(shutdown(app));
            }
        })
        .invoke_handler(tauri::generate_handler![
            plugins::operations::cancel_plugin_operation,
            plugins::commands::inspect_plugin_source,
            plugins::commands::inspect_plugin_update,
            plugins::commands::install_plugin,
            plugins::commands::update_plugin,
            plugins::commands::list_plugin_installations,
            plugins::commands::set_plugin_enabled,
            plugins::commands::uninstall_plugin,
            plugins::marketplaces::add_plugin_marketplace,
            plugins::marketplaces::refresh_plugin_marketplace,
            plugins::marketplaces::remove_plugin_marketplace,
            plugins::marketplaces::search_plugins,
            skills::commands::list_skills,
            skills::commands::read_skill,
            skills::commands::read_skill_file,
            skills::commands::create_skill,
            skills::commands::save_skill_revision,
            skills::commands::import_skill,
            skills::commands::export_skill,
            skills::commands::copy_skill,
            skills::commands::set_skill_enabled,
            skills::commands::delete_skill,
            mcp::browsing::read_mcp_resource,
            mcp::browsing::get_mcp_prompt,
            mcp::commands::create_mcp_connection,
            mcp::commands::update_mcp_connection,
            mcp::commands::list_mcp_connections,
            mcp::commands::check_mcp_connection,
            mcp::commands::delete_mcp_connection,
            connectors::commands::list_connector_accounts,
            connectors::definition_commands::list_connector_definitions,
            connectors::definition_commands::create_connector_definition,
            connectors::definition_commands::update_connector_definition,
            connectors::definition_commands::delete_connector_definition,
            connectors::commands::create_connector_account,
            connectors::commands::authorize_connector_account,
            connectors::commands::cancel_connector_authorization,
            connectors::commands::disconnect_connector_account,
            availability::get_agent_extension_availability,
        ])
        .build()
}

async fn shutdown(app: &tauri::AppHandle) {
    let state = app.state::<ExtensionState>();
    state.shutdown.cancel();
    mcp::sessions::shutdown(app).await;
    state.oauth_clients.lock().await.clear();
}

pub(crate) async fn recover(
    app: &tauri::AppHandle,
    retained_snapshots: &[String],
) -> Result<(), String> {
    let pool = database::pool(app).await?;
    connectors::commands::recover_cleanup(app).await?;
    connectors::cleanup::collect(app).await?;
    let leases: Vec<String> = sqlx::query_scalar("SELECT snapshot_id FROM content_leases")
        .fetch_all(&pool)
        .await
        .map_err(|error| error.to_string())?;
    for id in leases
        .into_iter()
        .filter(|id| !retained_snapshots.contains(id))
    {
        sqlx::query("DELETE FROM content_leases WHERE snapshot_id=$1")
            .bind(id)
            .execute(&pool)
            .await
            .map_err(|error| error.to_string())?;
    }
    reclamation::collect(app).await
}

#[cfg(test)]
#[path = "tests/content.rs"]
mod content_tests;
