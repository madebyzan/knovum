use crate::spaces::db;
use sqlx::Row;

use super::job::{EnqueueEnrichmentJobRequest, EnrichmentJobState, STATUS_RUNNING};

pub(crate) async fn claim_job(
    pool: &sqlx::SqlitePool,
    id: &str,
    request: &EnqueueEnrichmentJobRequest,
    now: i64,
) -> Result<bool, String> {
    let executed = sqlx::query(
        "UPDATE spaces_enrichment_jobs SET status=$1,error=NULL,updated_at=$2,attempts=attempts+1 \
         WHERE id=$3 AND space_id=$4 AND node_id=$5 AND job_type=$6 AND source_revision=$7 \
           AND status IN ('pending','failed','awaiting_configuration')",
    )
    .bind(STATUS_RUNNING)
    .bind(now)
    .bind(id)
    .bind(&request.space_id)
    .bind(&request.node_id)
    .bind(request.job_type.as_str())
    .bind(&request.source_revision)
    .execute(pool)
    .await
    .map_err(db::sql_error)?;
    Ok(executed.rows_affected() == 1)
}

pub(crate) async fn defer_for_configuration(
    pool: &sqlx::SqlitePool,
    id: &str,
    error: String,
    now: i64,
) -> Result<bool, String> {
    let executed = sqlx::query(
        "UPDATE spaces_enrichment_jobs SET status=$1,error=$2,updated_at=$3 \
         WHERE id=$4 AND status IN ('pending','failed','awaiting_configuration') \
         AND job_type IN ('analyze_image','analyze_video_cover','analyze_document')",
    )
    .bind("awaiting_configuration")
    .bind(error)
    .bind(now)
    .bind(id)
    .execute(pool)
    .await
    .map_err(db::sql_error)?;
    Ok(executed.rows_affected() == 1)
}

pub(crate) async fn set_status(
    pool: &sqlx::SqlitePool,
    id: &str,
    from: &'static str,
    status: &str,
    error: Option<String>,
    now: i64,
    increment_attempts: bool,
) -> Result<bool, String> {
    let sql = format!(
        "UPDATE spaces_enrichment_jobs SET status=$1, error=$2, updated_at=$3, \
         attempts=attempts + CASE WHEN $4 THEN 1 ELSE 0 END \
         WHERE id=$5 AND status IN ({from})"
    );
    let executed = sqlx::query(&sql)
        .bind(status)
        .bind(error)
        .bind(now)
        .bind(increment_attempts)
        .bind(id)
        .execute(pool)
        .await
        .map_err(db::sql_error)?;
    Ok(executed.rows_affected() == 1)
}

/// A job left `running` by a process that died can never be claimed again, so startup returns those
/// rows to the queue instead of reporting a failure the user did not cause.
pub(crate) async fn reconcile_interrupted(pool: &sqlx::SqlitePool, now: i64) -> Result<(), String> {
    sqlx::query(
        "UPDATE spaces_enrichment_jobs SET status='pending', updated_at=$1 WHERE status='running'",
    )
    .bind(now)
    .execute(pool)
    .await
    .map_err(db::sql_error)?;
    Ok(())
}

pub(crate) async fn list_actionable(
    pool: &sqlx::SqlitePool,
    space_id: &str,
) -> Result<Vec<EnrichmentJobState>, String> {
    let rows = sqlx::query(
        "WITH ranked AS ( \
           SELECT id,space_id,node_id,job_type,status,source_revision,attempts,error,created_at,updated_at, \
             row_number() OVER ( \
               PARTITION BY space_id,node_id,job_type,source_revision ORDER BY updated_at DESC,id DESC \
             ) AS position \
           FROM spaces_enrichment_jobs WHERE space_id=$1 \
         ) \
         SELECT r.id,r.space_id,r.node_id,r.job_type,r.status,r.source_revision,r.attempts,r.error,r.created_at,r.updated_at,n.data \
         FROM ranked r JOIN spaces_nodes n ON n.space_id=r.space_id AND n.id=r.node_id AND n.deleted_at IS NULL \
         WHERE r.position=1 AND r.status IN ('pending','running','failed','awaiting_configuration') \
           AND (r.job_type!='analyze_video_cover' OR NULLIF(json_extract(n.data,'$.previewPath'),'') IS NOT NULL) \
         ORDER BY r.updated_at ASC",
    )
    .bind(space_id)
    .fetch_all(pool)
    .await
    .map_err(db::sql_error)?;
    let mut actionable = Vec::new();
    for row in rows {
        let data: String = row.try_get("data").map_err(db::sql_error)?;
        let data: serde_json::Value = serde_json::from_str(&data).map_err(db::json_error)?;
        let kind: String = row.try_get("job_type").map_err(db::sql_error)?;
        let kind: super::job::EnrichmentJobType =
            serde_json::from_value(serde_json::Value::String(kind)).map_err(db::json_error)?;
        let revision = super::revision::expected_revision(&data, kind)?;
        let job_revision: String = row.try_get("source_revision").map_err(db::sql_error)?;
        if revision.as_deref() == Some(job_revision.as_str()) {
            actionable.push(job_from_row(row)?);
        }
    }
    Ok(actionable)
}

fn job_from_row(row: sqlx::sqlite::SqliteRow) -> Result<EnrichmentJobState, String> {
    Ok(EnrichmentJobState {
        id: row.try_get("id").map_err(db::sql_error)?,
        space_id: row.try_get("space_id").map_err(db::sql_error)?,
        node_id: row.try_get("node_id").map_err(db::sql_error)?,
        job_type: row.try_get("job_type").map_err(db::sql_error)?,
        status: row.try_get("status").map_err(db::sql_error)?,
        source_revision: row.try_get("source_revision").map_err(db::sql_error)?,
        attempts: row.try_get("attempts").map_err(db::sql_error)?,
        error: row.try_get("error").map_err(db::sql_error)?,
        created_at: row.try_get("created_at").map_err(db::sql_error)?,
        updated_at: row.try_get("updated_at").map_err(db::sql_error)?,
    })
}
