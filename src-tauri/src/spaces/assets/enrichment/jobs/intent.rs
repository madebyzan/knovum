use serde_json::Value;
use sqlx::{Sqlite, Transaction};

use super::job::{EnrichmentJobType, STATUS_CANCELLED, STATUS_PENDING};

pub(crate) async fn synchronize_for_node(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    node_id: &str,
    data: &Value,
    now: i64,
) -> Result<(), String> {
    let Some(source_revision) = source_revision(data) else {
        return Ok(());
    };
    sqlx::query(
        "UPDATE spaces_enrichment_jobs SET status=$1,updated_at=$2 \
         WHERE space_id=$3 AND node_id=$4 AND source_revision<>$5 AND status IN ('pending','running','awaiting_configuration')",
    )
    .bind(STATUS_CANCELLED)
    .bind(now)
    .bind(space_id)
    .bind(node_id)
    .bind(source_revision)
    .execute(&mut **tx)
    .await
    .map_err(crate::spaces::db::sql_error)?;

    for job_type in required_jobs(data, source_revision) {
        enqueue(tx, space_id, node_id, source_revision, job_type, now).await?;
    }
    Ok(())
}

pub(super) fn source_revision(data: &Value) -> Option<&str> {
    if matches!(
        data.get("processingState").and_then(Value::as_str),
        Some("importing" | "processing")
    ) {
        return None;
    }
    data.get("sourceRevision")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
}

fn required_jobs(data: &Value, source_revision: &str) -> Vec<EnrichmentJobType> {
    match data.get("kind").and_then(Value::as_str) {
        Some("image") => (!visual_knowledge_is_current(data, source_revision))
            .then_some(EnrichmentJobType::AnalyzeImage)
            .into_iter()
            .collect(),
        Some("video") => video_jobs(data, source_revision),
        Some("audio") => (!knowledge_is_current(data, "mediaKnowledge", source_revision))
            .then_some(EnrichmentJobType::TranscribeMedia)
            .into_iter()
            .collect(),
        Some("file") => document_jobs(data, source_revision),
        _ => Vec::new(),
    }
}

fn video_jobs(data: &Value, source_revision: &str) -> Vec<EnrichmentJobType> {
    let mut jobs = Vec::new();
    if data
        .get("previewPath")
        .and_then(Value::as_str)
        .is_some_and(|path| !path.is_empty())
        && !visual_knowledge_is_current(data, source_revision)
    {
        jobs.push(EnrichmentJobType::AnalyzeVideoFirstFrame);
    }
    if !knowledge_is_current(data, "mediaKnowledge", source_revision) {
        jobs.push(EnrichmentJobType::TranscribeMedia);
    }
    jobs
}

fn document_jobs(data: &Value, source_revision: &str) -> Vec<EnrichmentJobType> {
    let extension = data
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
    if !knowledge_is_current(data, "documentCoverage", source_revision) {
        return vec![EnrichmentJobType::ParseDocument];
    }
    let pending = data["documentCoverage"]["occurrences"]
        .as_array()
        .is_some_and(|items| {
            items
                .iter()
                .any(|item| matches!(item["state"].as_str(), Some("pending" | "failed")))
        });
    if pending {
        vec![EnrichmentJobType::AnalyzeDocument]
    } else {
        Vec::new()
    }
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
) -> Result<(), String> {
    sqlx::query(
        "INSERT INTO spaces_enrichment_jobs(id,space_id,node_id,job_type,status,source_revision,attempts,error,created_at,updated_at) \
         SELECT $1,$2,$3,$4,$5,$6,0,NULL,$7,$7 \
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
    .execute(&mut **tx)
    .await
    .map_err(crate::spaces::db::sql_error)?;
    Ok(())
}
