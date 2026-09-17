use sqlx::FromRow;
use tauri::{AppHandle, Runtime};

use super::index_writer;
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
    let pending = sqlx::query_as::<_, PendingInvalidation>(
        "SELECT id,space_id,node_id FROM spaces_retrieval_invalidations \
         ORDER BY node_id IS NOT NULL,id",
    )
    .fetch_all(pool)
    .await
    .map_err(db::sql_error)?;
    let mut failures = Vec::new();
    for invalidation in pending {
        let result = apply(app, &invalidation).await;
        if let Err(error) = result {
            record_failure(pool, invalidation.id, &error).await?;
            failures.push(format!("{}: {error}", invalidation.id));
        } else {
            acknowledge(pool, &invalidation).await?;
        }
    }
    if failures.is_empty() {
        Ok(())
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

async fn apply<R: Runtime>(
    app: &AppHandle<R>,
    invalidation: &PendingInvalidation,
) -> Result<(), String> {
    if let Some(node_id) = &invalidation.node_id {
        index_writer::delete_node_embeddings(
            app,
            &invalidation.space_id,
            std::slice::from_ref(node_id),
        )
        .await
    } else {
        index_writer::delete_space_embeddings(app, &invalidation.space_id).await
    }
}

async fn acknowledge(
    pool: &sqlx::SqlitePool,
    invalidation: &PendingInvalidation,
) -> Result<(), String> {
    if invalidation.node_id.is_some() {
        sqlx::query("DELETE FROM spaces_retrieval_invalidations WHERE id=$1")
            .bind(invalidation.id)
            .execute(pool)
            .await
            .map_err(db::sql_error)?;
    } else {
        sqlx::query("DELETE FROM spaces_retrieval_invalidations WHERE space_id=$1")
            .bind(&invalidation.space_id)
            .execute(pool)
            .await
            .map_err(db::sql_error)?;
    }
    Ok(())
}

async fn record_failure(pool: &sqlx::SqlitePool, id: i64, error: &str) -> Result<(), String> {
    sqlx::query(
        "UPDATE spaces_retrieval_invalidations SET attempts=attempts+1,last_error=$1 WHERE id=$2",
    )
    .bind(error)
    .bind(id)
    .execute(pool)
    .await
    .map_err(db::sql_error)?;
    Ok(())
}

#[cfg(test)]
mod tests;
