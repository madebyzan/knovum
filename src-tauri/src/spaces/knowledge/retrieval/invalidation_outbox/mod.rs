use lancedb::Connection;
use std::collections::BTreeMap;

mod settlement;
const INVALIDATION_BATCH_SIZE: usize = 128;
use sqlx::{FromRow, Sqlite, Transaction};
use tauri::{AppHandle, Runtime};

use super::lancedb_store::{connect_retrieval_db, open_knowledge_table_if_exists};
use crate::spaces::db;

#[derive(FromRow)]
struct PendingInvalidation {
    id: i64,
    space_id: String,
    node_id: Option<String>,
}

pub(in crate::spaces) async fn enqueue_nodes(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    space_id: &str,
    node_ids: &[String],
    now: i64,
) -> Result<(), String> {
    for node_id in node_ids {
        sqlx::query(
            "INSERT OR IGNORE INTO spaces_retrieval_invalidations(space_id,node_id,created_at) \
             SELECT $1,$2,$3 WHERE NOT EXISTS(\
               SELECT 1 FROM spaces_retrieval_invalidations WHERE space_id=$1 AND node_id IS NULL\
             )",
        )
        .bind(space_id)
        .bind(node_id)
        .bind(now)
        .execute(&mut **tx)
        .await
        .map_err(db::sql_error)?;
    }
    Ok(())
}

pub(in crate::spaces) async fn enqueue_space(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    space_id: &str,
    now: i64,
) -> Result<(), String> {
    sqlx::query(
        "DELETE FROM spaces_retrieval_invalidations WHERE space_id=$1 AND node_id IS NOT NULL",
    )
    .bind(space_id)
    .execute(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    sqlx::query(
        "INSERT OR IGNORE INTO spaces_retrieval_invalidations(space_id,node_id,created_at) \
         VALUES($1,NULL,$2)",
    )
    .bind(space_id)
    .bind(now)
    .execute(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    Ok(())
}

pub(crate) async fn drain<R: Runtime>(
    app: &AppHandle<R>,
    pool: &sqlx::SqlitePool,
) -> Result<(), String> {
    let index = connect_retrieval_db(app).await?;
    drain_with_connection(&index, pool).await
}

pub(super) async fn drain_with_connection(
    index: &Connection,
    pool: &sqlx::SqlitePool,
) -> Result<(), String> {
    loop {
        let mut tx = db::begin_write(pool).await?;
        let outcome = drain_batch(index, &mut tx, None).await;
        tx.commit().await.map_err(db::sql_error)?;
        if outcome? < INVALIDATION_BATCH_SIZE {
            return Ok(());
        }
    }
}

pub(super) async fn drain_pending(
    index: &Connection,
    tx: &mut Transaction<'_, Sqlite>,
    space_id: Option<&str>,
) -> Result<(), String> {
    while drain_batch(index, tx, space_id).await? == INVALIDATION_BATCH_SIZE {}
    Ok(())
}

async fn drain_batch(
    index: &Connection,
    tx: &mut Transaction<'_, Sqlite>,
    space_id: Option<&str>,
) -> Result<usize, String> {
    let pending = sqlx::query_as::<_, PendingInvalidation>(
        "SELECT id,space_id,node_id FROM spaces_retrieval_invalidations \
         WHERE $1 IS NULL OR space_id=$1 ORDER BY node_id IS NOT NULL,id LIMIT $2",
    )
    .bind(space_id)
    .bind(INVALIDATION_BATCH_SIZE as i64)
    .fetch_all(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    let count = pending.len();
    if pending.is_empty() {
        return Ok(0);
    }
    let table = open_knowledge_table_if_exists(index).await;
    let mut by_space: BTreeMap<String, Vec<PendingInvalidation>> = BTreeMap::new();
    for invalidation in pending {
        by_space
            .entry(invalidation.space_id.clone())
            .or_default()
            .push(invalidation);
    }
    let mut failures = Vec::new();
    for (space, invalidations) in by_space {
        let outcome = match &table {
            Ok(table) => settlement::apply(table.as_ref(), &space, &invalidations).await,
            Err(error) => Err(error.clone()),
        };
        match outcome {
            Ok(()) => settlement::acknowledge(tx, &space, &invalidations).await?,
            Err(error) => {
                for invalidation in invalidations {
                    record_failure(tx, invalidation.id, &error).await?;
                }
                failures.push(format!("{space}: {error}"));
            }
        }
    }
    if failures.is_empty() {
        Ok(count)
    } else {
        Err(format!(
            "Retrieval invalidations remain pending: {}",
            failures.join("; ")
        ))
    }
}

pub(crate) async fn drain_and_report<R: Runtime>(app: &AppHandle<R>, pool: &sqlx::SqlitePool) {
    if let Err(error) = drain(app, pool).await {
        log::error!("{error}");
    }
}

async fn record_failure(
    tx: &mut Transaction<'_, Sqlite>,
    id: i64,
    error: &str,
) -> Result<(), String> {
    sqlx::query(
        "UPDATE spaces_retrieval_invalidations SET attempts=attempts+1,last_error=$1 WHERE id=$2",
    )
    .bind(error)
    .bind(id)
    .execute(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    Ok(())
}

#[cfg(test)]
mod tests;
