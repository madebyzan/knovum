use crate::spaces::workflows::capture::contracts::CaptureIssueReason;
use serde_json::Value;
use sqlx::{Row, Sqlite, Transaction};

use super::contracts::{CaptureIssue, CapturedInput, RecordedExecutionSpec};
use crate::ai::tasks::TaskTargetSelection;
use crate::spaces::{actions::input_selection::source_node_ids, db};

pub(super) struct VerifiedActionSpec {
    pub recorded: RecordedExecutionSpec,
    pub source_node_ids: Vec<String>,
}

pub(super) fn sources_match(recorded: &[String], inputs: &[CapturedInput]) -> bool {
    recorded.len() == inputs.len()
        && recorded
            .iter()
            .zip(inputs)
            .all(|(node_id, input)| node_id == &input.node_id)
}

pub(super) async fn node_kind(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    node_id: &str,
) -> Result<Option<String>, String> {
    sqlx::query_scalar(
        "SELECT kind FROM spaces_nodes WHERE id=$1 AND space_id=$2 AND deleted_at IS NULL",
    )
    .bind(node_id)
    .bind(space_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(db::sql_error)
}

pub(super) async fn completed_action_spec(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    run_id: &str,
    action_id: &str,
    action_version: u16,
    output_kind: &str,
) -> Result<Option<VerifiedActionSpec>, String> {
    let stored: Option<String> = sqlx::query_scalar(
        "SELECT input_snapshot FROM spaces_runs WHERE id=$1 AND home_space_id=$2
         AND kind='action' AND state='succeeded'",
    )
    .bind(run_id)
    .bind(space_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    let Some(stored) = stored else {
        return Ok(None);
    };
    let spec: Value = serde_json::from_str(&stored).map_err(db::json_error)?;
    if spec.get("actionId").and_then(Value::as_str) != Some(action_id)
        || spec.get("actionVersion").and_then(Value::as_u64) != Some(u64::from(action_version))
        || !spec.get("parameters").is_some_and(Value::is_object)
    {
        return Ok(None);
    }
    let Ok(definition) = crate::spaces::actions::require(action_id) else {
        return Ok(None);
    };
    if definition.version != action_version {
        return Ok(None);
    }
    if !matches!(
        definition.execution_mode,
        crate::spaces::actions::SpaceActionExecutionMode::Durable
    ) {
        return Ok(None);
    }
    let expected_kind = match definition.output_contract {
        crate::spaces::actions::SpaceActionOutputContract::Document => "markdown",
        crate::spaces::actions::SpaceActionOutputContract::Image => "image",
        crate::spaces::actions::SpaceActionOutputContract::Video => "video",
        crate::spaces::actions::SpaceActionOutputContract::Presentation
        | crate::spaces::actions::SpaceActionOutputContract::File => "file",
        _ => return Ok(None),
    };
    if output_kind != expected_kind {
        return Ok(None);
    }
    if crate::spaces::actions::validate_parameters(definition, &spec["parameters"]).is_err() {
        return Ok(None);
    }
    let Some(source_snapshot) = spec.get("inputs") else {
        return Ok(None);
    };
    let Ok(source_node_ids) = source_node_ids(source_snapshot) else {
        return Ok(None);
    };
    let target = if definition.routing_purpose.is_some() {
        let Some(value) = spec.pointer(&format!("/inputs/targets/{action_id}")) else {
            return Ok(None);
        };
        let Ok(target) = serde_json::from_value::<TaskTargetSelection>(value.clone()) else {
            return Ok(None);
        };
        Some(target)
    } else {
        None
    };
    Ok(Some(VerifiedActionSpec {
        recorded: RecordedExecutionSpec {
            parameters: spec["parameters"].clone(),
            target,
        },
        source_node_ids,
    }))
}

pub(super) async fn derivation_inputs(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    derivation_id: &str,
) -> Result<(Vec<CapturedInput>, Vec<CaptureIssue>), String> {
    let rows = sqlx::query(
        "SELECT i.node_id,n.kind,i.role,i.position,i.source_revision,i.revision_basis
         FROM spaces_knowledge_derivation_inputs i
         LEFT JOIN spaces_nodes n ON n.id=i.node_id AND n.space_id=$1 AND n.deleted_at IS NULL
         WHERE i.derivation_id=$2 ORDER BY i.position",
    )
    .bind(space_id)
    .bind(derivation_id)
    .fetch_all(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    let mut inputs = Vec::with_capacity(rows.len());
    let mut issues = Vec::new();
    for row in rows {
        let node_id: Option<String> = row.try_get("node_id").map_err(db::sql_error)?;
        let kind: Option<String> = row.try_get("kind").map_err(db::sql_error)?;
        match (node_id, kind) {
            (Some(node_id), Some(kind)) => inputs.push(CapturedInput {
                node_id,
                kind,
                role: super::contracts::CaptureInputRole::parse(
                    &row.try_get::<String, _>("role").map_err(db::sql_error)?,
                )?,
                position: row.try_get("position").map_err(db::sql_error)?,
                source_revision: row.try_get("source_revision").map_err(db::sql_error)?,
                revision_basis: crate::spaces::derivation::input_revision::RevisionBasis::parse(
                    &row.try_get::<String, _>("revision_basis")
                        .map_err(db::sql_error)?,
                )?,
            }),
            (Some(node_id), None) => issues.push(CaptureIssue {
                node_id,
                reason: CaptureIssueReason::SourceUnavailable,
            }),
            (None, _) => issues.push(CaptureIssue {
                node_id: derivation_id.to_owned(),
                reason: CaptureIssueReason::SourceIdentityUnavailable,
            }),
        }
    }
    Ok((inputs, issues))
}
