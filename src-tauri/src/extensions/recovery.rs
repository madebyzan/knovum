use tauri::{AppHandle, Manager};

use super::{connectors, database, reclamation, ExtensionState};

pub(crate) async fn ready(app: &AppHandle) -> Result<(), String> {
    let state = app.state::<ExtensionState>();
    state
        .recovery
        .get_or_init(|| async {
            crate::backup::ready(app).await?;
            let _guard = state.content_lock.lock().await;
            connectors::cleanup::collect(app).await
        })
        .await
        .clone()
}

pub(crate) async fn reconcile_snapshots(
    app: &AppHandle,
    retained_snapshots: &[String],
) -> Result<(), String> {
    ready(app).await?;
    let state = app.state::<ExtensionState>();
    let _guard = state.content_lock.lock().await;
    let pool = database::pool(app).await?;
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
