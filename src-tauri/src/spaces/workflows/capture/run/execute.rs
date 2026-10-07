use crate::app_error::AppError;
use std::collections::HashMap;

use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use sqlx::{Row, SqlitePool};

use super::BoundResource;
use crate::spaces::{
    db,
    runs::{AttemptClaim, StepOutcome},
    workflows::capture::contracts::{CandidateSource, CandidateStep},
};

pub(crate) async fn execute_step(
    pool: &SqlitePool,
    claim: &AttemptClaim,
) -> Result<StepOutcome, AppError> {
    let step: CandidateStep =
        serde_json::from_value(claim.configuration.clone()).map_err(db::json_error)?;
    let bindings: HashMap<String, Vec<BoundResource>> = serde_json::from_value(
        claim
            .policy_snapshot
            .get("bindings")
            .cloned()
            .ok_or("workflow-run-invalid: input bindings are missing")?,
    )
    .map_err(db::json_error)?;
    let mut ordered = step.inputs.iter().collect::<Vec<_>>();
    ordered.sort_by_key(|input| input.position);
    let mut sources = Vec::with_capacity(ordered.len());
    for binding in ordered {
        let resources = match &binding.source {
            CandidateSource::InputSlot { input_slot_id } => {
                let sources = bindings
                    .get(input_slot_id)
                    .ok_or("workflow-run-invalid: an input slot is unbound")?;
                if sources.len() != 1 {
                    return Err(
                        "workflow-run-invalid: scalar input slot has multiple resources".into(),
                    );
                }
                sources.clone()
            }
            CandidateSource::InputSequence { input_slot_id } => bindings
                .get(input_slot_id)
                .ok_or("workflow-run-invalid: an input sequence is unbound")?
                .clone(),
            CandidateSource::StepOutput { step_id } => {
                vec![prior_output(pool, claim, step_id).await?]
            }
        };
        for resource in resources {
            verify_bound(pool, &claim.home_space_id, &resource).await?;
            sources.push(json!({"nodeId": resource.node_id, "role": binding.role, "revisionBasis": binding.revision_basis, "dataDigest": resource.data_digest}));
        }
    }
    let mut input = json!({"orderedInputs": sources});
    if let Some(target) = &step.execution_spec.target {
        input["targets"] = json!({&step.action_id: target});
    }
    crate::spaces::workflows::execution::invoke_captured(
        pool,
        claim,
        step.action_id,
        step.action_version,
        step.execution_spec.parameters,
        input,
    )
    .await
}

async fn verify_bound(
    pool: &SqlitePool,
    space_id: &str,
    source: &BoundResource,
) -> Result<(), String> {
    let row = sqlx::query(
        "SELECT kind,data FROM spaces_nodes WHERE id=$1 AND space_id=$2 AND deleted_at IS NULL",
    )
    .bind(&source.node_id)
    .bind(space_id)
    .fetch_optional(pool)
    .await
    .map_err(db::sql_error)?
    .ok_or("workflow-run-stale: input resource is unavailable")?;
    let kind: String = row.try_get("kind").map_err(db::sql_error)?;
    let data: String = row.try_get("data").map_err(db::sql_error)?;
    if kind != source.kind || hex::encode(Sha256::digest(data.as_bytes())) != source.data_digest {
        return Err("workflow-run-stale: input resource changed after the Run started".into());
    }
    Ok(())
}

async fn prior_output(
    pool: &SqlitePool,
    claim: &AttemptClaim,
    step_id: &str,
) -> Result<BoundResource, String> {
    let row = sqlx::query(
        "SELECT output,configuration FROM spaces_run_steps WHERE run_id=$1
         AND definition_node_id=$2 AND state='succeeded'",
    )
    .bind(&claim.run_id)
    .bind(step_id)
    .fetch_optional(pool)
    .await
    .map_err(db::sql_error)?
    .ok_or("workflow-run-invalid: upstream step has not succeeded")?;
    let configuration: String = row.try_get("configuration").map_err(db::sql_error)?;
    let upstream: CandidateStep = serde_json::from_str(&configuration).map_err(db::json_error)?;
    let output: String = row.try_get("output").map_err(db::sql_error)?;
    let output: Value = serde_json::from_str(&output).map_err(db::json_error)?;
    let child_run_id = output
        .get("childRunId")
        .and_then(Value::as_str)
        .ok_or("workflow-run-invalid: upstream action has no completed child Run")?;
    let resource = sqlx::query(
        "SELECT d.output_node_id,n.kind,n.data,n.updated_at,d.completed_at FROM spaces_knowledge_derivations d
         JOIN spaces_nodes n ON n.id=d.output_node_id AND n.space_id=d.space_id
         WHERE d.id=$1 AND d.space_id=$2 AND d.status='completed' AND n.deleted_at IS NULL",
    )
    .bind(child_run_id)
    .bind(&claim.home_space_id)
    .fetch_optional(pool)
    .await
    .map_err(db::sql_error)?
    .ok_or("workflow-run-invalid: upstream output is unavailable")?;
    let kind: String = resource.try_get("kind").map_err(db::sql_error)?;
    let updated_at: i64 = resource.try_get("updated_at").map_err(db::sql_error)?;
    let completed_at: i64 = resource.try_get("completed_at").map_err(db::sql_error)?;
    if kind != upstream.output_kind || updated_at > completed_at {
        return Err("workflow-run-stale: upstream output was edited after generation".into());
    }
    let node_id: String = resource.try_get("output_node_id").map_err(db::sql_error)?;
    let data: String = resource.try_get("data").map_err(db::sql_error)?;
    Ok(BoundResource {
        node_id,
        kind,
        data_digest: hex::encode(Sha256::digest(data.as_bytes())),
    })
}
