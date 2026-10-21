use serde::{de::IntoDeserializer, Deserialize};
use sqlx::Row;

use super::job::{
    EnqueueEnrichmentJobRequest, EnrichmentJobProjection, EnrichmentJobState, STATUS_RUNNING,
};
use crate::spaces::db;

#[derive(Clone, Copy, PartialEq, Eq)]
enum ArchivedSpaces {
    Include,
    Exclude,
}

pub(crate) async fn freeze_target(
    pool: &sqlx::SqlitePool,
    job_id: &str,
    target: &crate::ai::tasks::TaskTargetSelection,
) -> Result<crate::ai::tasks::TaskTargetSelection, String> {
    let frozen: Option<sqlx::types::Json<crate::ai::tasks::TaskTargetSelection>> = sqlx::query_scalar("UPDATE spaces_enrichment_jobs SET execution_target=COALESCE(execution_target,$1) WHERE id=$2 AND status IN ('pending','awaiting_configuration') RETURNING execution_target")
        .bind(sqlx::types::Json(target)).bind(job_id).fetch_optional(pool).await.map_err(db::sql_error)?;
    Ok(frozen
        .ok_or("Enrichment intent no longer permits execution preparation")?
        .0)
}

pub(crate) async fn claim_job(
    pool: &sqlx::SqlitePool,
    job_id: &str,
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
    .bind(job_id)
    .bind(&request.space_id)
    .bind(&request.node_id)
    .bind(request.job_type.as_str())
    .bind(&request.source_revision)
    .execute(pool)
    .await
    .map_err(db::sql_error)?;
    crate::spaces::runs::announce_change();
    Ok(executed.rows_affected() == 1)
}

pub(crate) async fn defer_for_configuration(
    pool: &sqlx::SqlitePool,
    job_id: &str,
    error: String,
    now: i64,
) -> Result<bool, String> {
    let executed = sqlx::query(
        "UPDATE spaces_enrichment_jobs SET status=$1,error=$2,updated_at=$3 \
         WHERE id=$4 AND status IN ('pending','failed','awaiting_configuration') \
         AND job_type='analyze_visual'",
    )
    .bind("awaiting_configuration")
    .bind(error)
    .bind(now)
    .bind(job_id)
    .execute(pool)
    .await
    .map_err(db::sql_error)?;
    crate::spaces::runs::announce_change();
    Ok(executed.rows_affected() == 1)
}

/// Writes one guarded status transition. Attempt counting is not a status-write concern:
/// `claim_job` already increments `attempts` when it moves a job to running, so every caller
/// records its outcome without touching the counter.
pub(crate) async fn set_status(
    pool: &sqlx::SqlitePool,
    job_id: &str,
    from: &'static str,
    status: &str,
    error: Option<String>,
    now: i64,
) -> Result<bool, String> {
    let mut sql = sqlx::QueryBuilder::<sqlx::Sqlite>::new(
        "UPDATE spaces_enrichment_jobs SET status=$1, error=$2, updated_at=$3 \
         WHERE id=$4 AND status IN (",
    );
    sql.push(from).push(")");
    let executed = sql
        .build()
        .bind(status)
        .bind(error)
        .bind(now)
        .bind(job_id)
        .execute(pool)
        .await
        .map_err(db::sql_error)?;
    crate::spaces::runs::announce_change();
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
    space_id: Option<&str>,
) -> Result<Vec<EnrichmentJobState>, String> {
    Ok(query_actionable(pool, space_id, ArchivedSpaces::Include)
        .await?
        .into_iter()
        .map(|projection| projection.job)
        .collect())
}

pub(crate) async fn list_actionable_projection(
    pool: &sqlx::SqlitePool,
    space_id: Option<&str>,
) -> Result<Vec<EnrichmentJobProjection>, String> {
    query_actionable(pool, space_id, ArchivedSpaces::Exclude).await
}

async fn query_actionable(
    pool: &sqlx::SqlitePool,
    space_id: Option<&str>,
    archived_spaces: ArchivedSpaces,
) -> Result<Vec<EnrichmentJobProjection>, String> {
    let rows = sqlx::query(
        "WITH ranked AS ( \
           SELECT id,space_id,node_id,job_type,status,source_revision,execution_target,attempts,error,created_at,updated_at, \
             row_number() OVER ( \
               PARTITION BY space_id,node_id,job_type,source_revision ORDER BY updated_at DESC,id DESC \
             ) AS position \
           FROM spaces_enrichment_jobs WHERE ($1 IS NULL OR space_id=$1) \
         ) \
         SELECT r.id,r.space_id,r.node_id,r.job_type,r.status,r.source_revision,r.execution_target,r.attempts,r.error,r.created_at,r.updated_at, \
           s.title AS space_title,n.kind AS node_kind,json_extract(n.data,'$.title') AS node_title,n.data \
         FROM ranked r \
         JOIN spaces s ON s.id=r.space_id \
         JOIN spaces_nodes n ON n.space_id=r.space_id AND n.id=r.node_id AND n.deleted_at IS NULL \
         WHERE r.position=1 AND r.status IN ('pending','running','failed','awaiting_configuration') \
           AND ($2 OR s.archived_at IS NULL) \
         ORDER BY r.updated_at ASC",
    )
    .bind(space_id)
    .bind(archived_spaces == ArchivedSpaces::Include)
    .fetch_all(pool)
    .await
    .map_err(db::sql_error)?;
    let mut actionable = Vec::new();
    for row in rows {
        let data: String = row.try_get("data").map_err(db::sql_error)?;
        let data: serde_json::Value = serde_json::from_str(&data).map_err(db::json_error)?;
        let kind: String = row.try_get("job_type").map_err(db::sql_error)?;
        let kind: super::job::EnrichmentJobType =
            super::job::EnrichmentJobType::deserialize(kind.into_deserializer())
                .map_err(db::json_error)?;
        let revision = super::revision::expected_revision(&data, kind)?;
        let job_revision: String = row.try_get("source_revision").map_err(db::sql_error)?;
        if revision.as_deref() == Some(job_revision.as_str()) {
            actionable.push(EnrichmentJobProjection {
                job: job_from_row(&row, kind, job_revision)?,
                space_title: row.try_get("space_title").map_err(db::sql_error)?,
                node_title: row.try_get("node_title").map_err(db::sql_error)?,
                node_kind: row.try_get("node_kind").map_err(db::sql_error)?,
            });
        }
    }
    Ok(actionable)
}

fn job_from_row(
    row: &sqlx::sqlite::SqliteRow,
    job_type: super::job::EnrichmentJobType,
    source_revision: String,
) -> Result<EnrichmentJobState, String> {
    Ok(EnrichmentJobState {
        id: row.try_get("id").map_err(db::sql_error)?,
        space_id: row.try_get("space_id").map_err(db::sql_error)?,
        node_id: row.try_get("node_id").map_err(db::sql_error)?,
        job_type,
        status: super::job::EnrichmentJobStatus::deserialize(
            row.try_get::<String, _>("status")
                .map_err(db::sql_error)?
                .into_deserializer(),
        )
        .map_err(db::json_error)?,
        source_revision,
        attempts: row.try_get("attempts").map_err(db::sql_error)?,
        execution_target: row
            .try_get::<Option<String>, _>("execution_target")
            .map_err(db::sql_error)?
            .map(|raw| serde_json::from_str(&raw).map_err(db::json_error))
            .transpose()?,
        error: row.try_get("error").map_err(db::sql_error)?,
        created_at: row.try_get("created_at").map_err(db::sql_error)?,
        updated_at: row.try_get("updated_at").map_err(db::sql_error)?,
    })
}
