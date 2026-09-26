use crate::spaces::workflows::capture::types::CaptureIssueReason;
use std::collections::{HashMap, HashSet};

use sqlx::{Row, Sqlite, Transaction};

use super::lookup::{completed_action_spec, derivation_inputs, node_kind, sources_match};
use super::order::order_steps;
use super::types::{CaptureIssue, CapturePreview, CapturedStep};
use super::{boundaries, candidate, fingerprint, selection};
use crate::spaces::db;

pub(super) async fn in_transaction(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    selected_node_ids: Vec<String>,
) -> Result<CapturePreview, String> {
    if space_id.trim().is_empty()
        || selected_node_ids.is_empty()
        || selected_node_ids.len() > 128
        || selected_node_ids.iter().any(|id| id.trim().is_empty())
    {
        return Err("workflow-capture-invalid: select 1 to 128 resources in one Space".into());
    }
    let selected: HashSet<_> = selected_node_ids.iter().cloned().collect();
    if selected.len() != selected_node_ids.len() {
        return Err("workflow-capture-invalid: selected resources must be unique".into());
    }
    let mut pending = selected_node_ids;
    let mut visited = HashSet::new();
    let mut steps = Vec::new();
    let mut boundaries = HashMap::new();
    let mut issues = Vec::new();

    while let Some(node_id) = pending.pop() {
        if !visited.insert(node_id.clone()) {
            continue;
        }
        if visited.len() > 256 {
            return Err(
                "workflow-capture-too-large: derivation contains more than 256 resources".into(),
            );
        }
        let Some(kind) = node_kind(tx, space_id, &node_id).await? else {
            issues.push(CaptureIssue {
                node_id,
                reason: CaptureIssueReason::ResourceUnavailable,
            });
            continue;
        };
        let derivation = sqlx::query(
            "SELECT id,action_kind,action_version,status FROM spaces_knowledge_derivations
             WHERE space_id=$1 AND output_node_id=$2",
        )
        .bind(space_id)
        .bind(&node_id)
        .fetch_optional(&mut **tx)
        .await
        .map_err(db::sql_error)?;
        let Some(derivation) = derivation else {
            boundaries::insert(&mut boundaries, node_id, kind, &selected);
            continue;
        };
        let derivation_id: String = derivation.try_get("id").map_err(db::sql_error)?;
        if derivation
            .try_get::<String, _>("status")
            .map_err(db::sql_error)?
            != "completed"
        {
            issues.push(CaptureIssue {
                node_id: node_id.clone(),
                reason: CaptureIssueReason::DerivationIncomplete,
            });
            boundaries::insert(&mut boundaries, node_id, kind, &selected);
            continue;
        }
        let action_id: String = derivation.try_get("action_kind").map_err(db::sql_error)?;
        let action_version = u16::try_from(
            derivation
                .try_get::<i64, _>("action_version")
                .map_err(db::sql_error)?,
        )
        .map_err(|_| "workflow-capture-invalid-action-version")?;
        let Some(spec) = completed_action_spec(
            tx,
            space_id,
            &derivation_id,
            &action_id,
            action_version,
            &kind,
        )
        .await?
        else {
            issues.push(CaptureIssue {
                node_id: node_id.clone(),
                reason: CaptureIssueReason::ExecutionSpecUnavailable,
            });
            boundaries::insert(&mut boundaries, node_id, kind, &selected);
            continue;
        };
        let (inputs, input_issues) = derivation_inputs(tx, space_id, &derivation_id).await?;
        if !input_issues.is_empty() {
            issues.extend(input_issues);
            boundaries::insert(&mut boundaries, node_id, kind, &selected);
            continue;
        }
        if inputs.is_empty() {
            issues.push(CaptureIssue {
                node_id: node_id.clone(),
                reason: CaptureIssueReason::DerivationInputsUnavailable,
            });
            boundaries::insert(&mut boundaries, node_id, kind, &selected);
            continue;
        }
        if !sources_match(&spec.source_node_ids, &inputs) {
            issues.push(CaptureIssue {
                node_id: node_id.clone(),
                reason: CaptureIssueReason::SourceMismatch,
            });
            boundaries::insert(&mut boundaries, node_id, kind, &selected);
            continue;
        }
        for input in &inputs {
            pending.push(input.node_id.clone());
        }
        steps.push(CapturedStep {
            derivation_id,
            output_node_id: node_id.clone(),
            output_kind: kind,
            action_id,
            action_version,
            execution_spec: spec.recorded,
            inputs,
            selected: selected.contains(&node_id),
        });
    }
    if steps.is_empty() && issues.is_empty() {
        issues.push(CaptureIssue {
            node_id: selected.iter().next().cloned().unwrap_or_default(),
            reason: CaptureIssueReason::NoDerivedOutputSelected,
        });
    }
    let steps = order_steps(steps)?;
    let selection_groups = selection::groups(&selected, &steps);
    let inputs = boundaries::finish(boundaries, &steps);
    let candidate = if issues.is_empty() && selection_groups.len() == 1 {
        match candidate::build(&steps, &inputs) {
            Ok(candidate) => Some(candidate),
            Err(reason) => {
                issues.push(CaptureIssue {
                    node_id: selection_groups[0][0].clone(),
                    reason,
                });
                None
            }
        }
    } else {
        None
    };
    let source_digest = if candidate.is_some() {
        Some(fingerprint::calculate(tx, space_id, &selected, &visited, &steps, &inputs).await?)
    } else {
        None
    };
    Ok(CapturePreview {
        steps,
        inputs,
        selection_groups,
        issues,
        candidate,
        source_digest,
    })
}
