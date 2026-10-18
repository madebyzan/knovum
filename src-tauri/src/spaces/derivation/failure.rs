use sqlx::{Sqlite, Transaction};
use tauri::AppHandle;

use super::pending_output;
use crate::spaces::db;

pub(super) fn with_settlement(
    primary: crate::foundation::app_error::AppError,
    settlement: Result<(), String>,
) -> crate::foundation::app_error::AppError {
    match settlement {
        Ok(()) => primary,
        Err(error) => crate::foundation::app_error::AppError {
            code: primary.code,
            message: format!(
                "{}; derivation failure settlement also failed: {error}",
                primary.message
            ),
        },
    }
}

pub(super) async fn fail_message(
    app: &AppHandle,
    derivation_id: &str,
    message: &str,
) -> Result<(), String> {
    let pool = db::pool(app).await?;
    fail_messages(pool, &[derivation_id.to_owned()], message).await
}

pub(super) async fn fail_messages(
    pool: &sqlx::SqlitePool,
    derivation_ids: &[String],
    message: &str,
) -> Result<(), String> {
    let mut tx = db::begin_write(pool).await?;
    for id in derivation_ids {
        mark_failed(&mut tx, id, message).await?;
    }
    tx.commit().await.map_err(db::sql_error)
}

/// Settles the output a Run reserved as failed, so the canvas stops showing work that is over.
pub async fn fail_pending(app: &AppHandle, node_id: &str) -> Result<(), String> {
    let pool = db::pool(app).await?;
    let mut tx = db::begin_write(pool).await?;
    pending_output::settle_failed(&mut tx, node_id).await?;
    tx.commit().await.map_err(db::sql_error)
}

pub(crate) async fn reconcile_interrupted(app: &AppHandle) -> Result<(), String> {
    let pool = db::pool(app).await?;
    let rows: Vec<(String, String)> = sqlx::query_as(
        "SELECT id,action_kind FROM spaces_knowledge_derivations WHERE status='running'",
    )
    .fetch_all(pool)
    .await
    .map_err(db::sql_error)?;
    for (id, action) in rows {
        let mut tx = db::begin_write(pool).await?;
        mark_failed(&mut tx, &id, &format!("{action} was interrupted")).await?;
        tx.commit().await.map_err(db::sql_error)?;
    }
    Ok(())
}

async fn mark_failed(
    tx: &mut Transaction<'_, Sqlite>,
    derivation_id: &str,
    error: &str,
) -> Result<(), String> {
    let now = crate::foundation::clock::unix_time_millis()?;
    let output_node_id: Option<String> = sqlx::query_scalar(
        "SELECT output_node_id FROM spaces_knowledge_derivations WHERE id=$1 AND status='running'",
    )
    .bind(derivation_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    let Some(node_id) = output_node_id else {
        return Ok(());
    };
    pending_output::settle_failed(tx, &node_id).await?;
    sqlx::query("UPDATE spaces_knowledge_derivations SET status='failed',error=$1,completed_at=$2,updated_at=$2 WHERE id=$3 AND status='running'")
        .bind(error).bind(now).bind(derivation_id).execute(&mut **tx).await.map_err(db::sql_error)?;
    Ok(())
}
