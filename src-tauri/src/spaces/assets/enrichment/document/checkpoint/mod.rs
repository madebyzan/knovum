use crate::spaces::knowledge::node_contracts::document_coverage::{
    DocumentCoverage, DocumentOccurrence, DocumentResource, VisualState,
};
use crate::spaces::{assets::enrichment::jobs::RunEnrichmentJobRequest, db};
use sqlx::Row;
use std::collections::BTreeMap;
use tauri::AppHandle;

/// Restore only this job's durable results against the unchanged logical input plan.
pub(super) async fn restore(
    app: &AppHandle,
    request: &RunEnrichmentJobRequest,
    report: &mut DocumentCoverage,
) -> Result<(), String> {
    let pool = db::pool(app).await?;
    let rows = sqlx::query("SELECT occurrence_id,occurrence,resource FROM spaces_document_visual_checkpoints WHERE job_id=$1")
        .bind(&request.job_id).fetch_all(pool).await.map_err(db::sql_error)?;
    let indices = report
        .occurrences
        .iter()
        .enumerate()
        .map(|(index, value)| (value.id.clone(), index))
        .collect::<BTreeMap<_, _>>();
    let mut resources = report
        .resources
        .iter()
        .cloned()
        .map(|value| (value.id.clone(), value))
        .collect::<BTreeMap<_, _>>();
    for row in rows {
        let id: String = row.try_get("occurrence_id").map_err(db::sql_error)?;
        let raw: String = row.try_get("occurrence").map_err(db::sql_error)?;
        let occurrence: DocumentOccurrence = serde_json::from_str(&raw).map_err(db::json_error)?;
        let index = *indices
            .get(&id)
            .ok_or("Document checkpoint input no longer exists")?;
        let raw: Option<String> = row.try_get("resource").map_err(db::sql_error)?;
        if let Some(raw) = raw {
            let resource: DocumentResource = serde_json::from_str(&raw).map_err(db::json_error)?;
            if occurrence.resource_id.as_deref() != Some(resource.id.as_str()) {
                return Err("Document checkpoint resource and occurrence disagree".into());
            }
            if resources
                .get(&resource.id)
                .is_some_and(|value| value != &resource)
            {
                return Err("Document checkpoint resource identity changed".into());
            }
            resources.insert(resource.id.clone(), resource);
        }
        restore_occurrence(&mut report.occurrences[index], occurrence)?;
    }
    report.resources = resources.into_values().collect();
    report.validate()
}

fn restore_occurrence(
    planned: &mut DocumentOccurrence,
    checkpoint: DocumentOccurrence,
) -> Result<(), String> {
    if checkpoint.id != planned.id
        || checkpoint.location != planned.location
        || checkpoint.pdf_input != planned.pdf_input
        || checkpoint.context != planned.context
        || checkpoint.alt_text != planned.alt_text
    {
        return Err("Document checkpoint and input plan disagree".into());
    }
    // Published completed knowledge is authoritative, including edits made before recovery.
    if !matches!(
        planned.state,
        VisualState::Completed | VisualState::Unsupported
    ) {
        *planned = checkpoint;
    }
    Ok(())
}

/// Write one occurrence and its raster reference, without rebuilding canonical knowledge.
pub(super) async fn save(
    app: &AppHandle,
    request: &RunEnrichmentJobRequest,
    report: &DocumentCoverage,
    index: usize,
) -> Result<(), String> {
    let occurrence = &report.occurrences[index];
    let resource = match &occurrence.resource_id {
        Some(id) => Some(
            report
                .resources
                .iter()
                .find(|value| &value.id == id)
                .ok_or("Document checkpoint raster is unavailable")?,
        ),
        None => None,
    };
    let pool = db::pool(app).await?;
    let checkpoint_write = sqlx::query("INSERT INTO spaces_document_visual_checkpoints(job_id,occurrence_id,occurrence,resource) \
        SELECT j.id,$1,$2,$3 FROM spaces_enrichment_jobs j JOIN spaces_nodes n ON n.id=j.node_id AND n.space_id=j.space_id \
        WHERE j.id=$4 AND j.space_id=$5 AND j.node_id=$6 AND j.source_revision=$7 AND j.job_type='analyze_document' \
          AND j.status='running' AND n.deleted_at IS NULL AND json_extract(n.data,'$.sourceRevision')=j.source_revision \
        ON CONFLICT(job_id,occurrence_id) DO UPDATE SET occurrence=excluded.occurrence,resource=excluded.resource")
        .bind(&occurrence.id)
        .bind(serde_json::to_string(occurrence).map_err(db::json_error)?)
        .bind(resource.map(serde_json::to_string).transpose().map_err(db::json_error)?)
        .bind(&request.job_id).bind(&request.space_id).bind(&request.node_id).bind(&report.source_revision)
        .execute(pool).await.map_err(db::sql_error)?;
    if checkpoint_write.rows_affected() != 1 {
        return Err("Document checkpoint job is no longer current".into());
    }
    Ok(())
}

pub(super) async fn clear(app: &AppHandle, job_id: &str) -> Result<(), String> {
    sqlx::query("DELETE FROM spaces_document_visual_checkpoints WHERE job_id=$1")
        .bind(job_id)
        .execute(db::pool(app).await?)
        .await
        .map_err(db::sql_error)?;
    Ok(())
}

#[cfg(test)]
mod tests;
