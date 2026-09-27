use std::collections::{HashMap, HashSet};

use serde::Deserialize;
use serde_json::Value;
use sqlx::SqlitePool;

use super::{
    library::SavedCapturedWorkflow,
    types::{CandidateOutputSlot, CandidateSource},
};
use crate::spaces::db;

#[derive(Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub(crate) struct ReviseCapturedWorkflowRequest {
    pub workflow_id: String,
    pub expected_version: u32,
    pub output_slots: Vec<CandidateOutputSlot>,
    #[ts(type = "Record<string, Record<string, unknown>>")]
    pub step_parameters: HashMap<String, Value>,
    pub removed_step_ids: Vec<String>,
}

pub(super) async fn revise(
    pool: &SqlitePool,
    request: ReviseCapturedWorkflowRequest,
) -> Result<SavedCapturedWorkflow, String> {
    let mut tx = db::begin_write(pool).await?;
    let definition: String = sqlx::query_scalar(
        "SELECT definition FROM spaces_workflow_templates WHERE id=$1
         AND json_extract(definition,'$.schemaVersion')=3",
    )
    .bind(&request.workflow_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(db::sql_error)?
    .ok_or("workflow-change-conflict: reload the workflow library")?;
    let mut saved: SavedCapturedWorkflow =
        serde_json::from_str(&definition).map_err(db::json_error)?;
    if saved.version != request.expected_version {
        return Err("workflow-change-conflict: reload the workflow library".into());
    }
    let previous = serde_json::to_value(&saved.candidate).map_err(db::json_error)?;
    let removed = request.removed_step_ids.into_iter().collect::<HashSet<_>>();
    if removed.len() > saved.candidate.steps.len()
        || removed
            .iter()
            .any(|id| !saved.candidate.steps.iter().any(|step| &step.id == id))
    {
        return Err("workflow-change-invalid: removed step is unavailable".into());
    }
    saved
        .candidate
        .steps
        .retain(|step| !removed.contains(&step.id));
    if request.step_parameters.len() != saved.candidate.steps.len() {
        return Err("workflow-change-invalid: edit each retained step exactly once".into());
    }
    let used_slots = saved
        .candidate
        .steps
        .iter()
        .flat_map(|step| &step.inputs)
        .filter_map(|binding| match &binding.source {
            CandidateSource::InputSlot { input_slot_id }
            | CandidateSource::InputSequence { input_slot_id } => Some(input_slot_id.as_str()),
            CandidateSource::StepOutput { .. } => None,
        })
        .collect::<HashSet<_>>();
    saved
        .candidate
        .input_slots
        .retain(|slot| used_slots.contains(slot.id.as_str()));
    saved.candidate.output_slots = request.output_slots;
    for output in &mut saved.candidate.output_slots {
        output.name = output.name.trim().to_owned();
    }
    for step in &mut saved.candidate.steps {
        step.execution_spec.parameters = request
            .step_parameters
            .get(&step.id)
            .ok_or("workflow-change-invalid: step is missing")?
            .clone();
    }
    super::run::validate_candidate(&saved.candidate)?;
    if previous == serde_json::to_value(&saved.candidate).map_err(db::json_error)? {
        return Err("workflow-change-invalid: no changes to save".into());
    }
    saved.version = saved
        .version
        .checked_add(1)
        .ok_or("workflow-change-limit: version overflow")?;
    let now = crate::clock::unix_time_millis()?;
    let encoded = serde_json::to_string(&saved).map_err(db::json_error)?;
    let workflow_write = sqlx::query(
        "UPDATE spaces_workflow_templates SET definition=$1,updated_at=$2 WHERE id=$3
         AND json_extract(definition,'$.version')=$4",
    )
    .bind(&encoded)
    .bind(now)
    .bind(&saved.workflow_id)
    .bind(i64::from(request.expected_version))
    .execute(&mut *tx)
    .await
    .map_err(db::sql_error)?;
    db::require_one(
        workflow_write,
        "workflow-change-conflict: reload the workflow library",
    )?;
    sqlx::query(
        "INSERT INTO spaces_captured_workflow_versions(workflow_id,version,definition,created_at)
         VALUES($1,$2,$3,$4)",
    )
    .bind(&saved.workflow_id)
    .bind(i64::from(saved.version))
    .bind(encoded)
    .bind(now)
    .execute(&mut *tx)
    .await
    .map_err(db::sql_error)?;
    tx.commit().await.map_err(db::sql_error)?;
    Ok(saved)
}

pub(super) async fn rename(
    pool: &SqlitePool,
    workflow_id: &str,
    expected_name: &str,
    name: &str,
) -> Result<SavedCapturedWorkflow, String> {
    let name = name.trim();
    if name.is_empty() || name.chars().count() > 120 {
        return Err("workflow-name-invalid: enter a name of at most 120 characters".into());
    }
    let mut tx = db::begin_write(pool).await?;
    let definition: String = sqlx::query_scalar(
        "SELECT definition FROM spaces_workflow_templates WHERE id=$1
         AND json_extract(definition,'$.schemaVersion')=3 AND name=$2",
    )
    .bind(workflow_id)
    .bind(expected_name)
    .fetch_optional(&mut *tx)
    .await
    .map_err(db::sql_error)?
    .ok_or("workflow-change-conflict: reload the workflow library")?;
    let mut saved: SavedCapturedWorkflow =
        serde_json::from_str(&definition).map_err(db::json_error)?;
    saved.name = name.to_owned();
    sqlx::query(
        "UPDATE spaces_workflow_templates SET name=$1,definition=$2,updated_at=$3 WHERE id=$4",
    )
    .bind(name)
    .bind(serde_json::to_string(&saved).map_err(db::json_error)?)
    .bind(crate::clock::unix_time_millis()?)
    .bind(workflow_id)
    .execute(&mut *tx)
    .await
    .map_err(db::sql_error)?;
    tx.commit().await.map_err(db::sql_error)?;
    Ok(saved)
}

pub(super) async fn delete(
    pool: &SqlitePool,
    workflow_id: &str,
    expected_name: &str,
) -> Result<(), String> {
    let workflow_write = sqlx::query(
        "DELETE FROM spaces_workflow_templates WHERE id=$1 AND name=$2
         AND json_extract(definition,'$.schemaVersion')=3",
    )
    .bind(workflow_id)
    .bind(expected_name)
    .execute(pool)
    .await
    .map_err(db::sql_error)?;
    db::require_one(
        workflow_write,
        "workflow-change-conflict: reload the workflow library",
    )
}
