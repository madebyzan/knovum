use serde_json::Value;
use sqlx::{Sqlite, Transaction};

use super::job::{EnrichmentJobType, STATUS_CANCELLED, STATUS_PENDING};
use super::revision::{source_revision, visual_revision};

pub(crate) async fn synchronize_for_node(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    node_id: &str,
    payload: &Value,
    now: i64,
    target: Option<&crate::ai::tasks::TaskTargetSelection>,
) -> Result<(), String> {
    let Some(source_revision) = source_revision(payload) else {
        return Ok(());
    };
    let visual_revision = visual_revision(payload)?;
    sqlx::query(
        "UPDATE spaces_enrichment_jobs SET status=$1,updated_at=$2 \
         WHERE space_id=$3 AND node_id=$4 AND \
           ((job_type='analyze_visual' AND ($5 IS NULL OR source_revision<>$5)) OR (job_type!='analyze_visual' AND source_revision<>$6)) \
           AND status IN ('pending','running','awaiting_configuration')",
    )
    .bind(STATUS_CANCELLED)
    .bind(now)
    .bind(space_id)
    .bind(node_id)
    .bind(visual_revision.as_deref())
    .bind(source_revision)
    .execute(&mut **tx)
    .await
    .map_err(crate::spaces::db::sql_error)?;

    for job_type in required_jobs(payload, visual_revision.as_deref(), source_revision) {
        let revision = if job_type == EnrichmentJobType::AnalyzeVisual {
            visual_revision
                .as_deref()
                .expect("visual jobs require a prepared input revision")
        } else {
            source_revision
        };
        enqueue(tx, space_id, node_id, revision, job_type, now, target).await?;
    }
    Ok(())
}

fn required_jobs(
    payload: &Value,
    visual_revision: Option<&str>,
    source_revision: &str,
) -> Vec<EnrichmentJobType> {
    let mut jobs = match payload.get("kind").and_then(Value::as_str) {
        Some("video" | "audio") => media_jobs(payload, source_revision),
        Some("file") => document_jobs(payload, source_revision),
        _ => Vec::new(),
    };
    if visual_revision.is_some_and(|revision| !visual_knowledge_is_current(payload, revision)) {
        jobs.push(EnrichmentJobType::AnalyzeVisual);
    }
    jobs
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
    if !knowledge_is_current(payload, "documentCoverage", source_revision) {
        return vec![EnrichmentJobType::ParseDocument];
    }
    let occurrences = payload["documentCoverage"]["occurrences"].as_array();
    let missing_pdf_resources = occurrences.is_some_and(|occurrences| {
        occurrences.iter().any(|occurrence| {
            !occurrence["pdfInput"].is_null()
                && occurrence["resourceId"].is_null()
                && matches!(occurrence["state"].as_str(), Some("pending" | "failed"))
        })
    });
    if missing_pdf_resources {
        return vec![EnrichmentJobType::ParseDocument];
    }
    let pending = occurrences.is_some_and(|occurrences| {
        occurrences
            .iter()
            .any(|occurrence| matches!(occurrence["state"].as_str(), Some("pending" | "failed")))
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
    .bind(target.filter(|_| matches!(job_type, EnrichmentJobType::AnalyzeVisual | EnrichmentJobType::AnalyzeDocument | EnrichmentJobType::ParseDocument)).map(sqlx::types::Json))
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
        Err(error) if error.code == "not_configured" => Ok(None),
        Err(error) => Err(error.to_string()),
    }
}
