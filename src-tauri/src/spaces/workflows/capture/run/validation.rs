use std::collections::HashSet;

use super::super::types::{CandidateSource, CaptureCandidate};
use crate::spaces::actions::{self, SpaceActionExecutionMode, SpaceActionOutputContract};

pub(crate) fn candidate(definition: &CaptureCandidate) -> Result<(), String> {
    if definition.steps.is_empty()
        || definition.steps.len() > 128
        || definition.output_slots.is_empty()
    {
        return Err("workflow-definition-invalid: action steps and outputs are required".into());
    }
    let mut inputs = HashSet::new();
    for slot in &definition.input_slots {
        if slot.id.is_empty()
            || slot.kind.is_empty()
            || slot.revision_bases.is_empty()
            || slot.min_items == 0
            || slot.max_items < slot.min_items
            || slot.max_items > 64
            || (!slot.ordered && (slot.min_items != 1 || slot.max_items != 1))
            || (slot.ordered && slot.max_items == 1)
            || !inputs.insert(&slot.id)
        {
            return Err("workflow-definition-invalid: input slots must be unique and typed".into());
        }
    }
    let mut completed_steps = HashSet::new();
    for step in &definition.steps {
        if step.id.is_empty() || completed_steps.contains(&step.id) || step.inputs.is_empty() {
            return Err("workflow-definition-invalid: step identity or inputs are invalid".into());
        }
        let action = actions::require(&step.action_id)?;
        if action.version != step.action_version
            || !matches!(action.execution_mode, SpaceActionExecutionMode::Durable)
            || output_kind(action.output_contract) != Some(step.output_kind.as_str())
        {
            return Err("workflow-definition-invalid: action contract changed".into());
        }
        actions::validate_parameters(action, &step.execution_spec.parameters)?;
        if action.routing_purpose.is_some() != step.execution_spec.target.is_some() {
            return Err("workflow-definition-invalid: execution target is missing".into());
        }
        let mut positions = HashSet::new();
        for binding in &step.inputs {
            if !positions.insert(binding.position) {
                return Err(
                    "workflow-definition-invalid: step input position or revision basis is invalid"
                        .into(),
                );
            }
            match &binding.source {
                CandidateSource::InputSlot { input_slot_id }
                    if inputs.contains(input_slot_id)
                        && definition.input_slots.iter().any(|slot| {
                            slot.id == *input_slot_id
                                && !slot.ordered
                                && slot.revision_bases.contains(&binding.revision_basis)
                        }) => {}
                CandidateSource::InputSequence { input_slot_id }
                    if inputs.contains(input_slot_id)
                        && definition.input_slots.iter().any(|slot| {
                            slot.id == *input_slot_id
                                && slot.ordered
                                && slot.revision_bases.contains(&binding.revision_basis)
                                && action.collection_input.is_some_and(|collection| {
                                    collection.kind == slot.kind
                                        && collection.role == binding.role.as_str()
                                        && slot.max_items <= collection.max_items
                                })
                        }) => {}
                CandidateSource::StepOutput { step_id } if completed_steps.contains(step_id) => {}
                _ => {
                    return Err(
                        "workflow-definition-invalid: step input has no earlier source".into(),
                    )
                }
            }
        }
        if positions.len() != step.inputs.len()
            || !positions.contains(&0)
            || positions
                .iter()
                .any(|position| *position < 0 || *position >= step.inputs.len() as i64)
            || step
                .inputs
                .iter()
                .find(|binding| binding.position == 0)
                .map(|binding| binding.role.as_str())
                != Some("primary")
        {
            return Err("workflow-definition-invalid: step input order is invalid".into());
        }
        completed_steps.insert(&step.id);
    }
    let used_inputs = definition
        .steps
        .iter()
        .flat_map(|step| &step.inputs)
        .filter_map(|binding| match &binding.source {
            CandidateSource::InputSlot { input_slot_id }
            | CandidateSource::InputSequence { input_slot_id } => Some(input_slot_id.as_str()),
            CandidateSource::StepOutput { .. } => None,
        })
        .collect::<HashSet<_>>();
    if used_inputs.len() != inputs.len() {
        return Err("workflow-definition-invalid: every input slot must be used".into());
    }
    let mut outputs = HashSet::new();
    let mut output_steps = HashSet::new();
    let mut output_names = HashSet::new();
    for output in &definition.output_slots {
        if output.id.is_empty()
            || !outputs.insert(&output.id)
            || !output_steps.insert(&output.step_id)
            || output.name.trim().is_empty()
            || output.name.chars().count() > 120
            || !output_names.insert(output.name.trim())
            || definition
                .steps
                .iter()
                .find(|step| step.id == output.step_id)
                .map(|step| step.output_kind.as_str())
                != Some(output.kind.as_str())
        {
            return Err("workflow-definition-invalid: output source is unavailable".into());
        }
    }
    Ok(())
}

fn output_kind(contract: SpaceActionOutputContract) -> Option<&'static str> {
    match contract {
        SpaceActionOutputContract::Document => Some("markdown"),
        SpaceActionOutputContract::Image => Some("image"),
        SpaceActionOutputContract::Video => Some("video"),
        SpaceActionOutputContract::Presentation | SpaceActionOutputContract::File => Some("file"),
        _ => None,
    }
}

pub(super) fn targets(app: &tauri::AppHandle, definition: &CaptureCandidate) -> Result<(), String> {
    for step in &definition.steps {
        let action = actions::require(&step.action_id)?;
        if let Some(target) = &step.execution_spec.target {
            let capability = action
                .execution_capability
                .ok_or("workflow-target-invalid: action does not accept an AI execution target")?;
            crate::ai::settings::resolve_task(app, target.clone(), capability)
                .map_err(|error| format!("workflow-target-unavailable: {error}"))?;
        }
    }
    Ok(())
}
