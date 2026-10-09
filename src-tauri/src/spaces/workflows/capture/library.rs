use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

use super::contracts::CaptureCandidate;
use crate::spaces::db;

#[derive(Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub(crate) struct SaveCapturedWorkflowRequest {
    pub request_id: String,
    pub space_id: String,
    pub selected_node_ids: Vec<String>,
    pub source_digest: String,
    pub name: String,
    pub output_names: std::collections::HashMap<String, String>,
}

#[derive(Clone, Deserialize, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub(crate) struct SavedCapturedWorkflow {
    pub schema_version: u16,
    pub workflow_id: String,
    pub version: u32,
    pub name: String,
    pub candidate: CaptureCandidate,
    pub source_space_id: String,
    pub source_digest: String,
    pub selected_node_ids: Vec<String>,
    pub created_at: i64,
}

#[derive(Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub(crate) struct CapturedWorkflowLibrary {
    pub items: Vec<SavedCapturedWorkflow>,
}

pub(super) async fn list(pool: &SqlitePool) -> Result<CapturedWorkflowLibrary, String> {
    let definitions: Vec<String> = sqlx::query_scalar(
        "SELECT definition FROM spaces_workflow_templates
         WHERE json_extract(definition,'$.schemaVersion')=1 ORDER BY updated_at DESC",
    )
    .fetch_all(pool)
    .await
    .map_err(db::sql_error)?;
    Ok(CapturedWorkflowLibrary {
        items: definitions
            .iter()
            .map(|definition| serde_json::from_str(definition).map_err(db::json_error))
            .collect::<Result<_, _>>()?,
    })
}

pub(super) async fn save(
    pool: &SqlitePool,
    request: SaveCapturedWorkflowRequest,
) -> Result<SavedCapturedWorkflow, String> {
    uuid::Uuid::parse_str(&request.request_id)
        .map_err(|_| "workflow-save-invalid: request ID must be a UUID")?;
    let name = request.name.trim();
    if name.is_empty() || name.chars().count() > 120 || request.source_digest.len() != 64 {
        return Err("workflow-save-invalid: name or source digest is invalid".into());
    }
    let mut tx = db::begin_write(pool).await?;
    let existing: Option<String> =
        sqlx::query_scalar("SELECT definition FROM spaces_workflow_templates WHERE id=$1")
            .bind(&request.request_id)
            .fetch_optional(&mut *tx)
            .await
            .map_err(db::sql_error)?;
    if let Some(existing) = existing {
        let saved: SavedCapturedWorkflow = serde_json::from_str(&existing)
            .map_err(|_| "workflow-save-conflict: request ID already belongs to another format")?;
        if saved.schema_version != 1
            || saved.source_digest != request.source_digest
            || saved.source_space_id != request.space_id
            || saved.selected_node_ids != request.selected_node_ids
            || saved.name != name
            || request.output_names.len() != saved.candidate.output_slots.len()
            || saved.candidate.output_slots.iter().any(|slot| {
                request.output_names.get(&slot.id).map(|name| name.trim())
                    != Some(slot.name.as_str())
            })
        {
            return Err("workflow-save-conflict: request ID was used for another workflow".into());
        }
        tx.commit().await.map_err(db::sql_error)?;
        return Ok(saved);
    }
    let preview = super::in_transaction(
        &mut tx,
        &request.space_id,
        request.selected_node_ids.clone(),
    )
    .await?;
    if preview.source_digest.as_deref() != Some(&request.source_digest) {
        return Err("workflow-save-stale: source changed; refresh the preview".into());
    }
    let mut candidate = preview
        .candidate
        .ok_or("workflow-save-invalid: selected derivation is not replayable")?;
    if request.output_names.len() != candidate.output_slots.len() {
        return Err("workflow-save-invalid: name each output exactly once".into());
    }
    for slot in &mut candidate.output_slots {
        slot.name = request
            .output_names
            .get(&slot.id)
            .ok_or("workflow-save-invalid: output name is missing")?
            .trim()
            .to_owned();
    }
    super::run::validate_candidate(&candidate)?;
    let now = crate::clock::unix_time_millis()?;
    let saved = SavedCapturedWorkflow {
        schema_version: 1,
        workflow_id: request.request_id,
        version: 1,
        name: name.to_owned(),
        candidate,
        source_space_id: request.space_id,
        source_digest: request.source_digest,
        selected_node_ids: request.selected_node_ids,
        created_at: now,
    };
    sqlx::query(
        "INSERT INTO spaces_workflow_templates(id,name,definition,created_at,updated_at)
         VALUES($1,$2,$3,$4,$4)",
    )
    .bind(&saved.workflow_id)
    .bind(&saved.name)
    .bind(sqlx::types::Json(&saved))
    .bind(now)
    .execute(&mut *tx)
    .await
    .map_err(db::sql_error)?;
    sqlx::query(
        "INSERT INTO spaces_captured_workflow_versions(workflow_id,version,definition,created_at)
         VALUES($1,$2,$3,$4)",
    )
    .bind(&saved.workflow_id)
    .bind(i64::from(saved.version))
    .bind(sqlx::types::Json(&saved))
    .bind(now)
    .execute(&mut *tx)
    .await
    .map_err(db::sql_error)?;
    tx.commit().await.map_err(db::sql_error)?;
    Ok(saved)
}
