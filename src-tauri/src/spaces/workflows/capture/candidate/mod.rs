use crate::spaces::workflows::capture::types::CaptureIssueReason;
use std::collections::{HashMap, HashSet};

use serde_json::Value;

use super::types::{
    BoundaryInput, CandidateBinding, CandidateInputSlot, CandidateOutputSlot, CandidateSource,
    CandidateStep, CaptureCandidate, CapturedStep,
};

mod collections;

pub(super) fn build(
    steps: &[CapturedStep],
    boundaries: &[BoundaryInput],
) -> Result<CaptureCandidate, CaptureIssueReason> {
    let source_ids = steps
        .iter()
        .flat_map(|step| step.inputs.iter().map(|input| input.node_id.as_str()))
        .chain(steps.iter().map(|step| step.output_node_id.as_str()))
        .collect::<HashSet<_>>();
    if steps
        .iter()
        .any(|step| contains_resource_reference(&step.execution_spec.parameters, &source_ids))
    {
        return Err(CaptureIssueReason::ParameterInstanceReference);
    }

    let by_boundary = boundaries
        .iter()
        .map(|input| (input.node_id.as_str(), input))
        .collect::<HashMap<_, _>>();
    let mut ordered_boundaries = Vec::new();
    let mut seen_boundaries = HashSet::new();
    for step in steps {
        for input in &step.inputs {
            if let Some(boundary) = by_boundary.get(input.node_id.as_str()) {
                if seen_boundaries.insert(input.node_id.as_str()) {
                    ordered_boundaries.push(*boundary);
                }
            }
        }
    }
    if ordered_boundaries.len() != boundaries.len() {
        return Err(CaptureIssueReason::CandidateBindingUnavailable);
    }
    let input_ids = ordered_boundaries
        .iter()
        .enumerate()
        .map(|(index, input)| (input.node_id.as_str(), format!("input-{}", index + 1)))
        .collect::<HashMap<_, _>>();
    let step_ids = steps
        .iter()
        .enumerate()
        .map(|(index, step)| (step.output_node_id.as_str(), format!("step-{}", index + 1)))
        .collect::<HashMap<_, _>>();

    let mut candidate_steps = Vec::with_capacity(steps.len());
    let mut outputs = Vec::new();
    for step in steps {
        let id = step_ids
            .get(step.output_node_id.as_str())
            .ok_or(CaptureIssueReason::CandidateBindingUnavailable)?
            .clone();
        let mut inputs = Vec::with_capacity(step.inputs.len());
        for input in &step.inputs {
            let source = if let Some(step_id) = step_ids.get(input.node_id.as_str()) {
                CandidateSource::StepOutput {
                    step_id: step_id.clone(),
                }
            } else {
                CandidateSource::InputSlot {
                    input_slot_id: input_ids
                        .get(input.node_id.as_str())
                        .ok_or(CaptureIssueReason::CandidateBindingUnavailable)?
                        .clone(),
                }
            };
            inputs.push(CandidateBinding {
                role: input.role.clone(),
                position: input.position,
                revision_basis: input.revision_basis.clone(),
                source,
            });
        }
        if step.selected {
            outputs.push(CandidateOutputSlot {
                id: format!("output-{}", outputs.len() + 1),
                name: format!("Output {}", outputs.len() + 1),
                kind: step.output_kind.clone(),
                step_id: id.clone(),
            });
        }
        candidate_steps.push(CandidateStep {
            id,
            action_id: step.action_id.clone(),
            action_version: step.action_version,
            output_kind: step.output_kind.clone(),
            execution_spec: step.execution_spec.clone(),
            inputs,
        });
    }
    if outputs.is_empty() {
        return Err(CaptureIssueReason::NoDerivedOutputSelected);
    }
    let mut candidate = CaptureCandidate {
        input_slots: ordered_boundaries
            .iter()
            .map(|input| CandidateInputSlot {
                id: input_ids
                    .get(input.node_id.as_str())
                    .expect("each boundary has an input slot")
                    .clone(),
                kind: input.kind.clone(),
                min_items: 1,
                max_items: 1,
                ordered: false,
                revision_bases: input
                    .uses
                    .iter()
                    .map(|use_| use_.revision_basis.clone())
                    .collect::<std::collections::BTreeSet<_>>()
                    .into_iter()
                    .collect(),
            })
            .collect(),
        steps: candidate_steps,
        output_slots: outputs,
    };
    collections::group(&mut candidate)?;
    Ok(candidate)
}

fn contains_resource_reference(value: &Value, source_ids: &HashSet<&str>) -> bool {
    match value {
        Value::String(text) => source_ids.contains(text.as_str()),
        Value::Array(items) => items
            .iter()
            .any(|item| contains_resource_reference(item, source_ids)),
        Value::Object(fields) => fields
            .values()
            .any(|item| contains_resource_reference(item, source_ids)),
        _ => false,
    }
}
