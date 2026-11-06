use std::collections::HashSet;

use serde_json::Value;
use sqlx::{Row, Sqlite, SqlitePool, Transaction};

use super::job::{EnrichmentJobType, STATUS_CANCELLED, STATUS_PENDING};
use super::recognition_source;
use super::revision::{expected_revision, source_revision, visual_revision};

pub(crate) async fn synchronize_for_node(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    node_id: &str,
    payload: &Value,
    now: i64,
    target: Option<&crate::ai::tasks::TaskTargetSelection>,
) -> Result<(), String> {
    if source_revision(payload).is_none() {
        return Ok(());
    }
    cancel_obsolete(tx, space_id, node_id, payload, now).await?;
    for (job_type, revision) in required_jobs(payload)? {
        enqueue(tx, space_id, node_id, &revision, job_type, now, target).await?;
    }
    Ok(())
}

/// A live job is obsolete once the input it was queued for is no longer the one its kind reads.
async fn cancel_obsolete(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    node_id: &str,
    payload: &Value,
    now: i64,
) -> Result<(), String> {
    let live = sqlx::query(
        "SELECT id,job_type,source_revision FROM spaces_enrichment_jobs \
         WHERE space_id=$1 AND node_id=$2 AND status IN ('pending','running','awaiting_configuration')",
    )
    .bind(space_id)
    .bind(node_id)
    .fetch_all(&mut **tx)
    .await
    .map_err(crate::spaces::db::sql_error)?;
    for row in live {
        let job_type: String = row
            .try_get("job_type")
            .map_err(crate::spaces::db::sql_error)?;
        let job_type: EnrichmentJobType = serde_json::from_value(Value::String(job_type))
            .map_err(crate::spaces::db::json_error)?;
        let revision: String = row
            .try_get("source_revision")
            .map_err(crate::spaces::db::sql_error)?;
        if expected_revision(payload, job_type)?.as_deref() == Some(revision.as_str()) {
            continue;
        }
        let id: String = row.try_get("id").map_err(crate::spaces::db::sql_error)?;
        sqlx::query("UPDATE spaces_enrichment_jobs SET status=$1,updated_at=$2 WHERE id=$3")
            .bind(STATUS_CANCELLED)
            .bind(now)
            .bind(id)
            .execute(&mut **tx)
            .await
            .map_err(crate::spaces::db::sql_error)?;
    }
    Ok(())
}

/// Every job the node owes, each with the revision of the input it reads.
fn required_jobs(payload: &Value) -> Result<Vec<(EnrichmentJobType, String)>, String> {
    let Some(source) = source_revision(payload) else {
        return Ok(Vec::new());
    };
    let mut jobs: Vec<_> = match payload.get("kind").and_then(Value::as_str) {
        Some("video" | "audio") => media_jobs(payload, source),
        Some("file") => document_jobs(payload, source),
        _ => Vec::new(),
    }
    .into_iter()
    .map(|job_type| (job_type, source.to_owned()))
    .collect();
    if let Some(revision) = recognition_due(payload)? {
        jobs.push((EnrichmentJobType::RecognizeText, revision));
    }
    if let Some(revision) =
        visual_revision(payload)?.filter(|revision| !visual_knowledge_is_current(payload, revision))
    {
        jobs.push((EnrichmentJobType::AnalyzeVisual, revision));
    }
    Ok(jobs)
}

fn media_jobs(data: &Value, source_revision: &str) -> Vec<EnrichmentJobType> {
    // Publishing technical analysis creates the transcription intent in the same node transaction.
    // A missing model or a failed/cancelled transcription can never withhold duration/waveform.
    if !knowledge_is_current(data, "mediaAnalysis", source_revision) {
        return vec![EnrichmentJobType::AnalyzeMedia];
    }
    (!knowledge_is_current(data, "mediaKnowledge", source_revision))
        .then_some(EnrichmentJobType::TranscribeMedia)
        .into_iter()
        .collect()
}

fn document_jobs(payload: &Value, source_revision: &str) -> Vec<EnrichmentJobType> {
    let extension = payload
        .get("fileName")
        .and_then(Value::as_str)
        .and_then(|name| std::path::Path::new(name).extension())
        .and_then(|value| value.to_str())
        .map(str::to_ascii_lowercase);
    if !matches!(
        extension.as_deref(),
        Some("pdf" | "doc" | "docx" | "xls" | "xlsx" | "ppt" | "pptx")
    ) {
        return Vec::new();
    }
    // Parsing extracts the text and every picture in one pass. Reading those pictures follows from
    // the coverage record it publishes, and cannot withhold the text it already extracted.
    (!knowledge_is_current(payload, "documentCoverage", source_revision))
        .then_some(EnrichmentJobType::ParseDocument)
        .into_iter()
        .collect()
}

/// The revision recognition owes the node, if it owes one.
fn recognition_due(payload: &Value) -> Result<Option<String>, String> {
    Ok(
        expected_revision(payload, EnrichmentJobType::RecognizeText)?.filter(|revision| {
            !knowledge_is_current(payload, "textRecognition", revision)
                && !recognition_source::sources(payload).is_empty()
        }),
    )
}

/// Queues recognition for nodes that predate it or whose intent was lost. A node with a job for its
/// revision, whatever became of that job, has been answered and is left alone.
pub(crate) async fn enqueue_missing_recognition(pool: &SqlitePool, now: i64) -> Result<(), String> {
    let rows = sqlx::query(
        "SELECT space_id,id,data FROM spaces_nodes \
         WHERE kind IN ('image','video','file') AND deleted_at IS NULL",
    )
    .fetch_all(pool)
    .await
    .map_err(crate::spaces::db::sql_error)?;
    let answered: HashSet<(String, String)> = sqlx::query_as(
        "SELECT node_id,source_revision FROM spaces_enrichment_jobs WHERE job_type='recognize_text'",
    )
    .fetch_all(pool)
    .await
    .map_err(crate::spaces::db::sql_error)?
    .into_iter()
    .collect();
    let mut tx = crate::spaces::db::begin_write(pool).await?;
    for row in rows {
        let raw: String = row.try_get("data").map_err(crate::spaces::db::sql_error)?;
        let payload: Value = serde_json::from_str(&raw).map_err(crate::spaces::db::json_error)?;
        let Some(revision) = recognition_due(&payload)? else {
            continue;
        };
        let node_id: String = row.try_get("id").map_err(crate::spaces::db::sql_error)?;
        if answered.contains(&(node_id.clone(), revision.clone())) {
            continue;
        }
        let space_id: String = row
            .try_get("space_id")
            .map_err(crate::spaces::db::sql_error)?;
        let job = EnrichmentJobType::RecognizeText;
        enqueue(&mut tx, &space_id, &node_id, &revision, job, now, None).await?;
    }
    tx.commit().await.map_err(crate::spaces::db::sql_error)
}

fn knowledge_is_current(data: &Value, field: &str, source_revision: &str) -> bool {
    data.get(field)
        .and_then(|knowledge| knowledge.get("sourceRevision"))
        .and_then(Value::as_str)
        == Some(source_revision)
}

fn visual_knowledge_is_current(data: &Value, source_revision: &str) -> bool {
    knowledge_is_current(data, "visualKnowledge", source_revision)
        && data
            .get("visualKnowledge")
            .and_then(|knowledge| knowledge.get("description"))
            .and_then(Value::as_str)
            .is_some_and(|description| !description.trim().is_empty())
}

async fn enqueue(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    node_id: &str,
    source_revision: &str,
    job_type: EnrichmentJobType,
    now: i64,
    target: Option<&crate::ai::tasks::TaskTargetSelection>,
) -> Result<(), String> {
    sqlx::query(
        "INSERT INTO spaces_enrichment_jobs(id,space_id,node_id,job_type,status,source_revision,execution_target,attempts,error,created_at,updated_at) \
         SELECT $1,$2,$3,$4,$5,$6,$8,0,NULL,$7,$7 \
         WHERE NOT EXISTS (SELECT 1 FROM spaces_enrichment_jobs \
           WHERE space_id=$2 AND node_id=$3 AND job_type=$4 AND source_revision=$6 AND status='failed') \
         ON CONFLICT(space_id,node_id,job_type,source_revision) WHERE status IN ('pending','running','awaiting_configuration') \
         DO UPDATE SET updated_at=excluded.updated_at",
    )
    .bind(uuid::Uuid::new_v4().to_string())
    .bind(space_id)
    .bind(node_id)
    .bind(job_type.as_str())
    .bind(STATUS_PENDING)
    .bind(source_revision)
    .bind(now)
    .bind(target.filter(|_| job_type == EnrichmentJobType::AnalyzeVisual).map(sqlx::types::Json))
    .execute(&mut **tx)
    .await
    .map_err(crate::spaces::db::sql_error)?;
    Ok(())
}

#[cfg(test)]
mod tests;

/// Capture policy at the node mutation boundary; execution validates runtime readiness later.
pub(in crate::spaces) fn selection<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
) -> Result<Option<crate::ai::tasks::TaskTargetSelection>, String> {
    match crate::ai::settings::default_selection(
        app,
        crate::ai::contracts::RoutingPurpose::VisualUnderstanding,
    ) {
        Ok(target) => Ok(Some(target)),
        Err(error) if error.is_not_configured() => Ok(None),
        Err(error) => Err(error.to_string()),
    }
}
