use serde::Serialize;
use serde_json::Value;
use tauri::{AppHandle, Runtime};

use super::super::{
    db,
    package::{PackageAsset, PACKAGE_VERSION},
};

mod derivations;
mod knowledge_rows;
mod package_assets;
mod rows;
mod workflow_runs;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SpacePackageSnapshot {
    manifest: Value,
    assets: Vec<PackageAsset>,
}

#[tauri::command]
pub async fn collect_space_package_snapshot<R: Runtime>(
    app: AppHandle<R>,
    space_id: String,
) -> Result<SpacePackageSnapshot, String> {
    let pool = db::pool(&app).await?;
    let mut tx = pool.begin().await.map_err(db::sql_error)?;
    let exported_at = crate::clock::unix_time_millis()?;

    let space = rows::space(&mut tx, &space_id).await?;
    let scenes = rows::scenes(&mut tx, &space_id).await?;
    let relations = rows::relations(&mut tx, &space_id).await?;
    let derivations = derivations::rows(&mut tx, &space_id).await?;
    let creation_workflow_runs = workflow_runs::rows(&mut tx, &space_id).await?;
    let mut assets = package_assets::PackageAssets::default();
    let nodes = rows::nodes(&mut tx, &space_id, &mut assets).await?;
    let canvas_state = rows::canvas_state(&mut tx, &space_id).await?;
    let chunks = rows::chunks(&mut tx, &space_id).await?;
    let ai_threads = rows::threads(&mut tx, &space_id).await?;
    let ai_messages = rows::messages(&mut tx, &space_id).await?;
    let object_types = knowledge_rows::object_types(&mut tx, &space_id).await?;
    let provenance = knowledge_rows::provenance(&mut tx, &space_id).await?;
    let mut entities = std::collections::HashSet::new();
    for (kind, rows) in [
        ("node", &nodes),
        ("relation", &relations),
        ("provenance", &provenance),
        ("type", &object_types),
    ] {
        for row in rows {
            let id = row
                .get("id")
                .and_then(Value::as_str)
                .ok_or("Export entity identity is unavailable")?;
            entities.insert((kind.to_owned(), id.to_owned()));
        }
    }
    let knowledge_events = knowledge_rows::events(&mut tx, &space_id, &entities).await?;

    let manifest = serde_json::json!({
        "format": "knovum-space",
        "version": PACKAGE_VERSION,
        "exportedAt": exported_at,
        "space": space,
        "canvasState": canvas_state,
        "nodes": nodes,
        "scenes": scenes,
        "relations": relations,
        "derivations": derivations,
        "creationWorkflowRuns": creation_workflow_runs,
        "chunks": chunks,
        "aiThreads": ai_threads,
        "aiMessages": ai_messages,
        "objectTypes": object_types,
        "provenance": provenance,
        "knowledgeEvents": knowledge_events,
    });
    tx.commit().await.map_err(db::sql_error)?;
    Ok(SpacePackageSnapshot {
        manifest,
        assets: assets.into_assets(),
    })
}

pub(super) fn parse_json(row: &sqlx::sqlite::SqliteRow, key: &str) -> Result<Value, String> {
    let raw: String = sqlx::Row::try_get(row, key).map_err(db::sql_error)?;
    serde_json::from_str(&raw).map_err(db::json_error)
}

pub(super) async fn reject_running(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    table: &str,
    space_id: &str,
    error_msg: &'static str,
) -> Result<(), String> {
    let query = format!("SELECT COUNT(*) FROM {table} WHERE space_id=$1 AND status='running'");
    let running: i64 = sqlx::query_scalar(&query)
        .bind(space_id)
        .fetch_one(&mut **tx)
        .await
        .map_err(db::sql_error)?;
    if running > 0 {
        return Err(error_msg.into());
    }
    Ok(())
}
