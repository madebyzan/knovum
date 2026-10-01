use lancedb::Connection;
use sqlx::{FromRow, Sqlite, Transaction};
use tauri::{AppHandle, Runtime};

use super::{
    lancedb_store::{connect_retrieval_db, open_knowledge_table_if_exists},
    predicates::{build_delete_node_predicate, build_delete_space_predicate},
};
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
    let mut tx = db::begin_write(pool).await?;
    let outcome = drain_pending(index, &mut tx, None).await;
    tx.commit().await.map_err(db::sql_error)?;
    outcome
}

pub(super) async fn drain_pending(
    index: &Connection,
    tx: &mut Transaction<'_, Sqlite>,
    space_id: Option<&str>,
) -> Result<(), String> {
    let pending = sqlx::query_as::<_, PendingInvalidation>(
        "SELECT id,space_id,node_id FROM spaces_retrieval_invalidations \
         WHERE $1 IS NULL OR space_id=$1 ORDER BY node_id IS NOT NULL,id",
    )
    .bind(space_id)
    .fetch_all(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    let mut failures = Vec::new();
    for invalidation in pending {
        let invalidation_outcome = apply(index, &invalidation).await;
        if let Err(error) = invalidation_outcome {
            record_failure(tx, invalidation.id, &error).await?;
            failures.push(format!("{}: {error}", invalidation.id));
        } else {
            acknowledge(tx, &invalidation).await?;
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

async fn apply(index: &Connection, invalidation: &PendingInvalidation) -> Result<(), String> {
    let Some(table) = open_knowledge_table_if_exists(index).await? else {
        return Ok(());
    };
    let predicate = match &invalidation.node_id {
        Some(node_id) => {
            build_delete_node_predicate(&invalidation.space_id, std::slice::from_ref(node_id))
        }
        None => build_delete_space_predicate(&invalidation.space_id),
    };
    table
        .delete(predicate.as_str())
        .await
        .map_err(|error| format!("Failed to delete LanceDB knowledge index rows: {error}"))?;
    Ok(())
}

async fn acknowledge(
    tx: &mut Transaction<'_, Sqlite>,
    invalidation: &PendingInvalidation,
) -> Result<(), String> {
    sqlx::query(
        "DELETE FROM spaces_chunk_embeddings WHERE chunk_id IN (\
         SELECT id FROM spaces_knowledge_chunks WHERE space_id=$1 AND ($2 IS NULL OR node_id=$2))",
    )
    .bind(&invalidation.space_id)
    .bind(&invalidation.node_id)
    .execute(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    if invalidation.node_id.is_some() {
        sqlx::query("DELETE FROM spaces_retrieval_invalidations WHERE id=$1")
            .bind(invalidation.id)
            .execute(&mut **tx)
            .await
            .map_err(db::sql_error)?;
    } else {
        sqlx::query("DELETE FROM spaces_retrieval_invalidations WHERE space_id=$1")
            .bind(&invalidation.space_id)
            .execute(&mut **tx)
            .await
            .map_err(db::sql_error)?;
    }
    Ok(())
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
