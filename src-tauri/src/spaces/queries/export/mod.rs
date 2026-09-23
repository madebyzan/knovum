use serde::Serialize;
use serde_json::Value;
use tauri::{AppHandle, Runtime};

use super::super::{
    db,
    package::{PackageAsset, PACKAGE_VERSION},
};

mod assistant;
mod automations;
mod derivations;
mod external_conversations;
mod external_tools;
mod knowledge_rows;
mod package_assets;
mod rows;
mod runs;
mod workflows;

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
    reject_active_runs(&mut tx, &space_id).await?;
    let workflow_drafts = workflows::drafts(&mut tx, &space_id).await?;
    let workflow_versions = workflows::versions(&mut tx, &space_id).await?;
    let (runs, run_steps, run_batch_items, run_attempts, run_events) =
        runs::rows(&mut tx, &space_id).await?;
    let (automation_triggers, automation_occurrences) =
        automations::rows(&mut tx, &space_id).await?;
    let external_tool_bindings = external_tools::rows(&mut tx, &space_id).await?;
    let (external_agent_conversations, external_agent_turns, external_agent_messages) =
        external_conversations::rows(&mut tx, &space_id).await?;
    let (assistant_conversations, assistant_messages) = assistant::rows(&mut tx, &space_id).await?;
    let mut assets = package_assets::PackageAssets::default();
    let nodes = rows::nodes(&mut tx, &space_id, &mut assets).await?;
    let (mut structure, _) =
        crate::spaces::structure::repository::load_in_transaction(&mut tx, &space_id).await?;
    rows::package_layout_assets(&mut structure.layouts, &mut assets)?;
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
        "structureNodes": structure.structure,
        "layouts": structure.layouts,
        "relationStyles": structure.relation_styles,
        "nodes": nodes,
        "scenes": scenes,
        "relations": relations,
        "derivations": derivations,
        "workflowDrafts": workflow_drafts,
        "workflowVersions": workflow_versions,
        "runs": runs,
        "runSteps": run_steps,
        "runBatchItems": run_batch_items,
        "runAttempts": run_attempts,
        "runEvents": run_events,
        "automationTriggers": automation_triggers,
        "automationOccurrences": automation_occurrences,
        "externalToolBindings": external_tool_bindings,
        "externalAgentConversations": external_agent_conversations,
        "externalAgentTurns": external_agent_turns,
        "externalAgentMessages": external_agent_messages,
        "assistantConversations": assistant_conversations,
        "assistantMessages": assistant_messages,
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

async fn reject_active_runs(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    space_id: &str,
) -> Result<(), String> {
    let running: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM spaces_runs WHERE home_space_id=$1 AND completed_at IS NULL",
    )
    .bind(space_id)
    .fetch_one(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    if running > 0 {
        return Err("Active Runs must finish or be cancelled before export".into());
    }
    Ok(())
}
