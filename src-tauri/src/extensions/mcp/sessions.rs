use super::super::{
    contracts::{AgentExtensionSnapshot, ConnectorDefinition, McpConnection},
    ExtensionState,
};
use super::{
    client::{self, McpSession},
    McpInteraction,
};
use std::{collections::HashMap, sync::Arc};
use tauri::{AppHandle, Manager};
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

pub(in crate::extensions) struct SessionSlot {
    connection: McpConnection,
    plugin_id: Option<String>,
    cancellation: CancellationToken,
    session: Mutex<Option<Arc<McpSession>>>,
}
pub(in crate::extensions) type Sessions = HashMap<(String, String), Arc<SessionSlot>>;

async fn register(
    sessions: &Mutex<Sessions>,
    shutdown: &CancellationToken,
    scope: &str,
    connection: &McpConnection,
    connector: &ConnectorDefinition,
) -> Arc<SessionSlot> {
    sessions
        .lock()
        .await
        .entry((scope.into(), connection.id.clone()))
        .or_insert_with(|| {
            Arc::new(SessionSlot {
                connection: connection.clone(),
                plugin_id: connector.plugin_id.clone(),
                cancellation: shutdown.child_token(),
                session: Mutex::new(None),
            })
        })
        .clone()
}

pub(super) async fn with_inspection<T, F, Fut>(
    app: &AppHandle,
    connection: &McpConnection,
    connector: &ConnectorDefinition,
    operation: F,
) -> Result<T, String>
where
    F: FnOnce(Arc<McpSession>) -> Fut,
    Fut: std::future::Future<Output = Result<T, String>>,
{
    super::connections::require_reviewable(app, connection).await?;
    let pool = super::super::database::pool(app).await?;
    let account = match &connection.account_id {
        Some(id) => Some(super::super::connectors::accounts::load(&pool, id).await?),
        None => None,
    };
    let scope = uuid::Uuid::new_v4().to_string();
    let state = app.state::<ExtensionState>();
    let slot = register(
        &state.sessions,
        &state.shutdown,
        &scope,
        connection,
        connector,
    )
    .await;
    let inspect = async {
        let session = Arc::new(
            client::connect(
                app,
                connection,
                connector,
                None,
                slot.cancellation.child_token(),
            )
            .await?,
        );
        *slot.session.lock().await = Some(session.clone());
        super::connections::require_reviewable(app, connection).await?;
        super::super::connectors::definitions::require_current(&pool, connector).await?;
        if let Some(account) = &account {
            super::super::connectors::accounts::require_current(&pool, account).await?;
        }
        let value = operation(session).await?;
        super::connections::require_reviewable(app, connection).await?;
        super::super::connectors::definitions::require_current(&pool, connector).await?;
        if let Some(account) = &account {
            super::super::connectors::accounts::require_current(&pool, account).await?;
        }
        Ok(value)
    };
    let inspection = tokio::select! {
        biased;
        _ = slot.cancellation.cancelled() => Err("mcp-inspection-session-revoked".into()),
        inspection = inspect => inspection,
    };
    release(app, &scope).await;
    inspection
}

pub(crate) async fn acquire(
    app: &AppHandle,
    snapshot: &AgentExtensionSnapshot,
    connection: &McpConnection,
    interaction: Option<Arc<dyn McpInteraction>>,
) -> Result<Arc<McpSession>, String> {
    super::connections::require_current(app, connection).await?;
    let connector = snapshot
        .connectors
        .iter()
        .find(|connector| connector.id == connection.connector_id)
        .ok_or("snapshot-connector-definition-missing")?;
    super::super::connectors::definitions::require_current(
        &super::super::database::pool(app).await?,
        connector,
    )
    .await?;
    let state = app.state::<ExtensionState>();
    let slot = register(
        &state.sessions,
        &state.shutdown,
        &snapshot.id,
        connection,
        connector,
    )
    .await;
    let mut stored = slot.session.lock().await;
    if slot.cancellation.is_cancelled() {
        return Err("mcp-task-session-revoked".into());
    }
    if let Some(session) = stored.as_ref() {
        if session.service.is_closed() {
            return Err("mcp-task-session-closed".into());
        }
        return Ok(session.clone());
    }
    let connected = async {
        let session = Arc::new(
            client::connect(
                app,
                connection,
                connector,
                interaction,
                slot.cancellation.child_token(),
            )
            .await?,
        );
        super::connections::require_current(app, connection).await?;
        super::super::connectors::definitions::require_current(
            &super::super::database::pool(app).await?,
            connector,
        )
        .await?;
        if let Some(expected) = snapshot.catalogs.get(&connection.id) {
            let current = super::catalog::read(&session.service).await?;
            if current.digest != expected.digest {
                return Err("mcp-server-catalog-changed-start-new-task".into());
            }
        }
        Ok::<_, String>(session)
    };
    let session = tokio::select! {
        biased;
        _ = slot.cancellation.cancelled() => return Err("mcp-task-session-revoked".into()),
        connected = connected => connected?,
    };
    *stored = Some(session.clone());
    Ok(session)
}

pub(crate) async fn inspect(
    app: &AppHandle,
    snapshot: &AgentExtensionSnapshot,
    connection: &McpConnection,
    interaction: Option<Arc<dyn McpInteraction>>,
) -> Result<super::super::contracts::McpConnectionCatalog, String> {
    let session = acquire(app, snapshot, connection, interaction).await?;
    super::catalog::read(&session.service).await
}

async fn revoke(app: &AppHandle, predicate: impl Fn(&(String, String), &SessionSlot) -> bool) {
    revoke_matching(&app.state::<ExtensionState>().sessions, predicate).await;
}

async fn revoke_matching(
    sessions: &Mutex<Sessions>,
    predicate: impl Fn(&(String, String), &SessionSlot) -> bool,
) {
    let removed = {
        let mut sessions = sessions.lock().await;
        let keys: Vec<_> = sessions
            .iter()
            .filter(|(key, slot)| predicate(key, slot))
            .map(|(key, _)| key.clone())
            .collect();
        keys.into_iter()
            .filter_map(|key| sessions.remove(&key))
            .collect::<Vec<_>>()
    };
    for slot in removed {
        slot.cancellation.cancel();
        if let Some(session) = slot.session.lock().await.take() {
            session.cancel();
            if let Ok(session) = Arc::try_unwrap(session) {
                session.close().await;
            }
        }
    }
}

pub(crate) async fn release(app: &AppHandle, snapshot_id: &str) {
    revoke(app, |(id, _), _| id == snapshot_id).await;
}
pub(in crate::extensions) async fn shutdown(app: &AppHandle) {
    revoke(app, |_, _| true).await;
}
pub(crate) async fn revoke_connection(app: &AppHandle, connection_id: &str) {
    revoke(app, |(_, id), _| id == connection_id).await;
}
pub(crate) async fn revoke_account(app: &AppHandle, account_id: &str) {
    revoke(app, |_, slot| {
        slot.connection.account_id.as_deref() == Some(account_id)
    })
    .await;
    app.state::<ExtensionState>()
        .oauth_clients
        .lock()
        .await
        .remove(account_id);
}
pub(crate) async fn revoke_plugin(app: &AppHandle, plugin_id: &str) {
    revoke(app, |_, slot| slot.plugin_id.as_deref() == Some(plugin_id)).await;
}

pub(in crate::extensions) async fn revoke_connector(app: &AppHandle, connector_id: &str) {
    revoke(app, |_, slot| slot.connection.connector_id == connector_id).await;
}

#[cfg(test)]
#[path = "tests/sessions.rs"]
mod tests;
