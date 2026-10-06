use crate::app_error::AppError;
use sqlx::{Row, Sqlite, SqlitePool, Transaction};

use crate::spaces::db;

pub(crate) fn optional_limit(budget: &serde_json::Value, key: &str) -> Result<Option<i64>, String> {
    match budget.get(key) {
        None => Ok(None),
        Some(value) => value
            .as_i64()
            .filter(|limit| *limit >= 0)
            .map(Some)
            .ok_or_else(|| format!("run-budget-invalid: {key} must be a nonnegative integer")),
    }
}

pub(crate) async fn record_usage(
    pool: &SqlitePool,
    run_id: &str,
    usage: &crate::ai::contracts::TokenUsage,
) -> Result<(), String> {
    let mut tx = db::begin_write(pool).await?;
    let within_budget = record_tokens_in_transaction(
        &mut tx,
        run_id,
        i64::from(usage.prompt_tokens),
        i64::from(usage.completion_tokens),
        i64::from(usage.total_tokens),
    )
    .await?;
    // Paid usage remains recorded even when it exceeds the admission budget.
    tx.commit().await.map_err(db::sql_error)?;
    if !within_budget {
        return Err("budget-exhausted: Run tree model token budget was exceeded".into());
    }
    Ok(())
}

pub(crate) async fn record_tokens_in_transaction(
    tx: &mut Transaction<'_, Sqlite>,
    run_id: &str,
    input: i64,
    output: i64,
    total: i64,
) -> Result<bool, String> {
    let recorded: Vec<(String, i64)> = sqlx::query_as(
        "WITH RECURSIVE ancestors(id,parent_run_id) AS (
           SELECT id,parent_run_id FROM spaces_runs WHERE id=$1
           UNION ALL SELECT parent.id,parent.parent_run_id FROM spaces_runs parent
           JOIN ancestors child ON child.parent_run_id=parent.id)
         UPDATE spaces_runs SET input_tokens=input_tokens+$2,output_tokens=output_tokens+$3,
           total_tokens=total_tokens+$4,updated_at=$5 WHERE id IN (SELECT id FROM ancestors)
         RETURNING budget,total_tokens",
    )
    .bind(run_id)
    .bind(input)
    .bind(output)
    .bind(total)
    .bind(crate::clock::unix_time_millis()?)
    .fetch_all(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    if recorded.is_empty() {
        return Err("run-unavailable: usage owner is unavailable".into());
    }
    let mut within_budget = true;
    for (budget, total) in recorded {
        let budget = serde_json::from_str(&budget).map_err(db::json_error)?;
        within_budget &=
            optional_limit(&budget, "maxTotalTokens")?.is_none_or(|limit| total <= limit);
    }
    Ok(within_budget)
}

pub(crate) async fn reserve_output_bytes_in_transaction(
    tx: &mut Transaction<'_, Sqlite>,
    run_id: &str,
    bytes: usize,
    now: i64,
) -> Result<(), AppError> {
    let bytes = i64::try_from(bytes)
        .map_err(|_| AppError::new("budget_exhausted", "Output is too large"))?;
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
        let limit = optional_limit(&budget, "maxOutputBytes")?.unwrap_or(i64::MAX);
        if used.saturating_add(bytes) > limit {
            return Err(AppError::new(
                "budget_exhausted",
                "Run tree output byte budget was exceeded",
            ));
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

#[cfg(test)]
mod tests;
