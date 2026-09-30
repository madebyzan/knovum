use sqlx::{Row, Sqlite, SqlitePool, Transaction};

use crate::spaces::db;

pub(crate) async fn record_usage(
    pool: &SqlitePool,
    run_id: &str,
    usage: &crate::ai::contracts::TokenUsage,
) -> Result<(), String> {
    let mut tx = db::begin_write(pool).await?;
    let ancestors = sqlx::query_scalar::<_, String>(
        "WITH RECURSIVE ancestors(id,parent_run_id) AS (
           SELECT id,parent_run_id FROM spaces_runs WHERE id=$1
           UNION ALL
           SELECT parent.id,parent.parent_run_id FROM spaces_runs parent
           JOIN ancestors child ON child.parent_run_id=parent.id
         ) SELECT id FROM ancestors",
    )
    .bind(run_id)
    .fetch_all(&mut *tx)
    .await
    .map_err(db::sql_error)?;
    if ancestors.is_empty() {
        return Err("run-unavailable: usage owner is unavailable".into());
    }
    let now = crate::clock::unix_time_millis()?;
    for ancestor in ancestors {
        let budget_write = sqlx::query(
            "UPDATE spaces_runs SET input_tokens=input_tokens+$1,output_tokens=output_tokens+$2,
             total_tokens=total_tokens+$3,updated_at=$4 WHERE id=$5 AND completed_at IS NULL
             AND total_tokens+$3 <= COALESCE(json_extract(budget,'$.maxTotalTokens'),9223372036854775807)",
        ).bind(i64::from(usage.prompt_tokens)).bind(i64::from(usage.completion_tokens))
            .bind(i64::from(usage.total_tokens)).bind(now).bind(ancestor)
            .execute(&mut *tx).await.map_err(db::sql_error)?;
        db::require_one(
            budget_write,
            "budget-exhausted: Run tree model token budget was exceeded",
        )?;
    }
    tx.commit().await.map_err(db::sql_error)
}

pub(crate) async fn reserve_output_bytes_in_transaction(
    tx: &mut Transaction<'_, Sqlite>,
    run_id: &str,
    bytes: usize,
    now: i64,
) -> Result<(), String> {
    let bytes = i64::try_from(bytes).map_err(|_| "budget-exhausted: output is too large")?;
    let rows = sqlx::query(
        "WITH RECURSIVE ancestors(id,parent_run_id,output_bytes,budget) AS (
           SELECT id,parent_run_id,output_bytes,budget FROM spaces_runs WHERE id=$1
           UNION ALL
           SELECT parent.id,parent.parent_run_id,parent.output_bytes,parent.budget
           FROM spaces_runs parent JOIN ancestors child ON child.parent_run_id=parent.id
         ) SELECT id,output_bytes,budget FROM ancestors",
    )
    .bind(run_id)
    .fetch_all(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    if rows.is_empty() {
        return Err("run-unavailable: output budget owner is unavailable".into());
    }
    let mut ids = Vec::with_capacity(rows.len());
    for row in rows {
        let used: i64 = row.try_get("output_bytes").map_err(db::sql_error)?;
        let budget: String = row.try_get("budget").map_err(db::sql_error)?;
        let budget: serde_json::Value = serde_json::from_str(&budget).map_err(db::json_error)?;
        let limit = budget
            .get("maxOutputBytes")
            .and_then(serde_json::Value::as_i64)
            .unwrap_or(i64::MAX);
        if used.saturating_add(bytes) > limit {
            return Err("budget-exhausted: Run tree output byte budget was exceeded".into());
        }
        ids.push(row.try_get::<String, _>("id").map_err(db::sql_error)?);
    }
    for id in ids {
        sqlx::query(
            "UPDATE spaces_runs SET output_bytes=output_bytes+$1,updated_at=$2 WHERE id=$3",
        )
        .bind(bytes)
        .bind(now)
        .bind(id)
        .execute(&mut **tx)
        .await
        .map_err(db::sql_error)?;
    }
    Ok(())
}
