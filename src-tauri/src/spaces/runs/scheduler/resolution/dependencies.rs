use sqlx::{Row, Sqlite, Transaction};

use super::{dependency_facts, dependency_input, join_policy, next_state, run_failure_policy};
use crate::spaces::db;

pub(crate) async fn resolve_dependents(
    tx: &mut Transaction<'_, Sqlite>,
    run_id: &str,
    now: i64,
) -> Result<(), String> {
    loop {
        let rows = sqlx::query("SELECT id,depends_on,node_kind,configuration FROM spaces_run_steps WHERE run_id=$1 AND state='pending'")
            .bind(run_id).fetch_all(&mut **tx).await.map_err(db::sql_error)?;
        let mut changed = false;
        for row in rows {
            changed |= resolve_dependent(tx, run_id, &row, now).await?
        }
        if !changed {
            break;
        }
    }
    Ok(())
}

async fn resolve_dependent(
    tx: &mut Transaction<'_, Sqlite>,
    run_id: &str,
    row: &sqlx::sqlite::SqliteRow,
    now: i64,
) -> Result<bool, String> {
    let id: String = row.try_get("id").map_err(db::sql_error)?;
    let encoded: String = row.try_get("depends_on").map_err(db::sql_error)?;
    let node_kind: String = row.try_get("node_kind").map_err(db::sql_error)?;
    let configuration: String = row.try_get("configuration").map_err(db::sql_error)?;
    let dependencies: Vec<String> = serde_json::from_str(&encoded).map_err(db::json_error)?;
    let facts = dependency_facts(tx, run_id, &dependencies).await?;
    let policy = run_failure_policy(tx, run_id).await?;
    let collect_results = node_kind == "join" && join_policy(&configuration)? == "collect_results";
    let Some(state) = next_state(
        &facts
            .iter()
            .map(|(state, _)| state.clone())
            .collect::<Vec<_>>(),
        collect_results || policy == "collect_errors",
    ) else {
        return Ok(false);
    };
    let input =
        (state == "ready").then(|| dependency_input(&dependencies, &facts, collect_results));
    let dependency_write = sqlx::query("UPDATE spaces_run_steps SET state=$1,resolved_input=COALESCE($2,resolved_input),state_version=state_version+1,updated_at=$3 WHERE id=$4 AND state='pending'")
        .bind(state).bind(input.as_ref().map(sqlx::types::Json)).bind(now).bind(&id).execute(&mut **tx).await.map_err(db::sql_error)?;
    Ok(dependency_write.rows_affected() == 1)
}
