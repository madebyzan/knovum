use tauri::AppHandle;

use crate::spaces::{conversations::tools, db, run_control};

pub(super) async fn retire(app: &AppHandle, pool: &sqlx::SqlitePool) -> Result<(), String> {
    let records: Vec<String> = sqlx::query_scalar("SELECT r.id FROM spaces_builtin_agent_tasks t JOIN spaces_runs r ON r.id=t.run_id
        JOIN spaces_builtin_agent_execution_segments s ON s.id=t.segment_id WHERE r.completed_at IS NULL
        AND (s.graph_version!=$1 OR json_extract(r.policy_snapshot,'$.builtinAgentProfileVersion') IS NOT $2)")
        .bind(crate::agent_runtime::protocol::GRAPH_VERSION).bind(i64::from(super::BUILTIN_AGENT_PROFILE_VERSION))
        .fetch_all(pool).await.map_err(db::sql_error)?;
    for run_id in records {
        cancel(
            app,
            pool,
            &run_id,
            "builtin-agent-graph-version-changed: start a new execution segment",
        )
        .await?;
    }
    let records: Vec<(String,String,String)> = sqlx::query_as("SELECT r.id,r.input_snapshot,t.catalog_digest FROM spaces_builtin_agent_tasks t JOIN spaces_runs r ON r.id=t.run_id WHERE r.completed_at IS NULL AND r.cancel_requested_at IS NULL")
        .fetch_all(pool).await.map_err(db::sql_error)?;
    for (run_id, input, expected) in records {
        let input: serde_json::Value = serde_json::from_str(&input).map_err(db::json_error)?;
        let snapshot: crate::extensions::contracts::AgentExtensionSnapshot =
            serde_json::from_value(input["extensions"].clone()).map_err(db::json_error)?;
        let mut catalog = tools::project()?;
        catalog.extend(&snapshot)?;
        if catalog.digest != expected {
            cancel(
                app,
                pool,
                &run_id,
                "builtin-agent-action-catalog-changed: start a new execution segment",
            )
            .await?;
        }
    }
    Ok(())
}

async fn cancel(
    app: &AppHandle,
    pool: &sqlx::SqlitePool,
    run_id: &str,
    error: &str,
) -> Result<(), String> {
    sqlx::query("UPDATE spaces_builtin_agent_tasks SET error=$1,updated_at=$2 WHERE run_id=$3")
        .bind(error)
        .bind(crate::clock::unix_time_millis()?)
        .bind(run_id)
        .execute(pool)
        .await
        .map_err(db::sql_error)?;
    let receipt = run_control::cancel_run(app.clone(), run_id.into())
        .await
        .map_err(|error| error.to_string())?;
    for error in receipt.interruption_errors {
        log::warn!("Retired Built-in Agent Run with pending interruption: {error}");
    }
    Ok(())
}
