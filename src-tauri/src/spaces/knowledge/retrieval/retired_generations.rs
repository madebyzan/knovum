use lancedb::{table::OptimizeAction, Connection};
use sqlx::{FromRow, SqlitePool};
use tauri::{AppHandle, Runtime};

use super::{lancedb_store, predicates::escape_sql_literal};
use crate::spaces::db;

static CLEANUP: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());
static PENDING: tokio::sync::Notify = tokio::sync::Notify::const_new();

const HISTORY_RETENTION_MS: i64 = 7 * 24 * 60 * 60 * 1000;
const CLEANUP_INTERVAL: std::time::Duration = std::time::Duration::from_secs(60 * 60);

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
    drain_with_connection(&index, pool, crate::clock::unix_time_millis()?).await
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
    if retired.is_empty() {
        return Ok(());
    }
    let table = lancedb_store::open_knowledge_table_if_exists(index).await?;
    for generation in &retired {
        if generation.cleanup_started_at.is_none() {
            if let Some(table) = &table {
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
    }
    // Retention starts after physical compaction, including a long maintenance pass.
    let completed_at = crate::clock::unix_time_millis()?.max(now);
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
