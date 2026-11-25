use lancedb::{index::Index, table::OptimizeAction, Connection, Table};
use sqlx::{FromRow, SqlitePool};
use tauri::{AppHandle, Runtime};

use super::{invalidation_outbox, lancedb_store, predicates::escape_sql_literal};
use crate::spaces::db;

static CLEANUP: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());
static PENDING: tokio::sync::Notify = tokio::sync::Notify::const_new();

const HISTORY_RETENTION_MS: i64 = 7 * 24 * 60 * 60 * 1000;
const CLEANUP_INTERVAL: std::time::Duration = std::time::Duration::from_secs(60 * 60);
const VECTOR_INDEX_MIN_ROWS: usize = 4_096;

#[derive(FromRow)]
struct RetiredGeneration {
    id: String,
    space_id: String,
    cleanup_started_at: Option<i64>,
}

pub(crate) fn start(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        loop {
            let outcome = async {
                let pool = db::pool(&app).await?;
                drain(&app, pool).await
            }
            .await;
            if let Err(error) = outcome {
                log::error!("Knowledge retired-generation cleanup remains pending: {error}");
            }
            tokio::select! {
                _ = tokio::time::sleep(CLEANUP_INTERVAL) => {},
                _ = PENDING.notified() => {},
            }
        }
    });
}

pub(super) fn notify_pending() {
    PENDING.notify_one();
}

pub(crate) async fn drain<R: Runtime>(app: &AppHandle<R>, pool: &SqlitePool) -> Result<(), String> {
    let index = lancedb_store::connect_retrieval_db(app).await?;
    drain_with_connection(&index, pool, crate::foundation::clock::unix_time_millis()?).await
}

pub(super) async fn drain_with_connection(
    index: &Connection,
    pool: &SqlitePool,
    now: i64,
) -> Result<(), String> {
    // Serialize this owner's maintenance without holding the UI database across physical compaction.
    // Index publication rejects retired generations, and LanceDB owns concurrent dataset commits.
    let _cleanup = CLEANUP.lock().await;
    let retired = sqlx::query_as::<_, RetiredGeneration>(
        "SELECT g.id,g.space_id,g.cleanup_started_at FROM spaces_retrieval_generations g \
         WHERE g.status='retired' AND (g.cleanup_started_at IS NULL OR g.cleanup_started_at<=$1) \
         AND EXISTS(SELECT 1 FROM spaces_retrieval_generations a WHERE a.space_id=g.space_id AND a.status='active') \
         ORDER BY g.created_at,g.id",
    ).bind(now.saturating_sub(HISTORY_RETENTION_MS)).fetch_all(pool).await.map_err(db::sql_error)?;
    let table = lancedb_store::open_knowledge_table_if_exists(index).await?;
    if retired.is_empty() {
        if let Some(table) = &table {
            maintain_active_table(table).await;
        }
        return Ok(());
    }
    for generation in &retired {
        if generation.cleanup_started_at.is_none() {
            if let Some(table) = &table {
                let _lance_guard = invalidation_outbox::acquire_lance_write_lock().await;
                let predicate = format!(
                    "space_id = '{}' AND generation_id = '{}'",
                    escape_sql_literal(&generation.space_id),
                    escape_sql_literal(&generation.id)
                );
                table
                    .delete(&predicate)
                    .await
                    .map_err(|error| format!("Failed to retire generation vectors: {error}"))?;
            }
            sqlx::query("DELETE FROM spaces_chunk_embeddings WHERE generation_id=$1")
                .bind(&generation.id)
                .execute(pool)
                .await
                .map_err(db::sql_error)?;
        }
    }
    if let Some(table) = &table {
        let _lance_guard = invalidation_outbox::acquire_lance_write_lock().await;
        // Materialize every deletion; keep the documented seven-day snapshot safety window.
        table
            .optimize(OptimizeAction::Compact {
                options: lancedb::table::CompactionOptions {
                    materialize_deletions_threshold: 0.0,
                    ..Default::default()
                },
                remap_options: None,
            })
            .await
            .map_err(|error| format!("Failed to compact retired generation data: {error}"))?;
        table
            .optimize(OptimizeAction::Prune {
                older_than: None,
                delete_unverified: Some(false),
                error_if_tagged_old_versions: Some(true),
            })
            .await
            .map_err(|error| format!("Failed to prune retired generation history: {error}"))?;
        ensure_table_indices(table).await;
    }
    // Retention starts after physical compaction, including a long maintenance pass.
    let completed_at = crate::foundation::clock::unix_time_millis()?.max(now);
    for generation in retired {
        if generation.cleanup_started_at.is_some() {
            sqlx::query(
                "DELETE FROM spaces_retrieval_generations WHERE id=$1 AND status='retired'",
            )
            .bind(&generation.id)
            .execute(pool)
            .await
            .map_err(db::sql_error)?;
            log::info!(
                "knowledge_index_cleanup generation={} event=completed",
                generation.id
            );
        } else {
            sqlx::query("UPDATE spaces_retrieval_generations SET cleanup_started_at=$1 WHERE id=$2 AND status='retired'")
                .bind(completed_at).bind(&generation.id).execute(pool).await.map_err(db::sql_error)?;
            log::info!(
                "knowledge_index_cleanup generation={} event=history_retention",
                generation.id
            );
        }
    }
    Ok(())
}

/// Periodically compacts incremental LanceDB writes, prunes expired history, and maintains
/// scalar/vector indices on the active table even when no generation has been retired.
async fn maintain_active_table(table: &Table) {
    let _lance_guard = invalidation_outbox::acquire_lance_write_lock().await;
    if let Err(error) = table
        .optimize(OptimizeAction::Compact {
            options: Default::default(),
            remap_options: None,
        })
        .await
    {
        log::warn!("Incremental LanceDB compaction skipped: {error}");
    }
    if let Err(error) = table
        .optimize(OptimizeAction::Prune {
            older_than: None,
            delete_unverified: Some(false),
            error_if_tagged_old_versions: Some(true),
        })
        .await
    {
        log::warn!("Incremental LanceDB prune skipped: {error}");
    }
    ensure_table_indices(table).await;
}

async fn ensure_table_indices(table: &Table) {
    let Ok(rows) = table.count_rows(None).await else {
        return;
    };
    if rows == 0 {
        return;
    }
    let Ok(indices) = table.list_indices().await else {
        return;
    };
    let has_space_index = indices
        .iter()
        .any(|index| index.columns.iter().any(|column| column == "space_id"));
    if !has_space_index {
        if let Err(error) = table
            .create_index(&["space_id"], Index::BTree(Default::default()))
            .execute()
            .await
        {
            log::warn!("Failed to build LanceDB space_id BTree index: {error}");
        }
    }
    let has_vector_index = indices
        .iter()
        .any(|index| index.columns.iter().any(|column| column == "vector"));
    if !has_vector_index && rows >= VECTOR_INDEX_MIN_ROWS {
        if let Err(error) = table.create_index(&["vector"], Index::Auto).execute().await {
            log::warn!("Failed to build LanceDB vector index: {error}");
        }
    }
}
