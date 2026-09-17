use std::collections::{HashMap, HashSet};

use serde::Deserialize;
use serde_json::Value;
use tauri::AppHandle;

use crate::{
    ai::{models::ExecutionTargetRef, settings},
    spaces::knowledge,
};

use super::{require, WorkflowDefinition, WorkflowParameterKind};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SavedWorkflow {
    id: String,
    name: String,
    workflow_id: String,
    workflow_version: i64,
    targets: HashMap<String, SavedTargetBinding>,
    parameters: Value,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct SavedTargetBinding {
    action_id: String,
    target: ExecutionTargetRef,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SavedGenerationPreset {
    id: String,
    name: String,
    action_id: String,
    target: ExecutionTargetRef,
    parameters: Value,
}

pub fn validate_presets(
    app: &AppHandle,
    presets: Vec<SavedGenerationPreset>,
) -> Result<(), String> {
    let mut ids = HashSet::new();
    for preset in presets {
        if preset.id.trim().is_empty()
            || preset.name.trim().is_empty()
            || !ids.insert(preset.id.clone())
        {
            return Err("Generation Preset metadata is invalid".into());
        }
        let action = knowledge::actions::require(&preset.action_id)?;
        knowledge::actions::validate_parameters(action, &preset.parameters)?;
        let capability = action
            .execution_capability
            .ok_or_else(|| "Generation Preset action has no execution target".to_string())?;
        settings::resolve(app, preset.target.with_temperature(0.2), capability)
            .map_err(|error| error.to_string())?;
    }
    Ok(())
}

pub fn validate_workflows(app: &AppHandle, workflows: Vec<SavedWorkflow>) -> Result<(), String> {
    let mut ids = HashSet::new();
    for workflow in workflows {
        if workflow.id.trim().is_empty()
            || workflow.name.trim().is_empty()
            || !ids.insert(workflow.id.clone())
        {
            return Err("Saved Workflow metadata is invalid".into());
        }
        let definition = require(&workflow.workflow_id, workflow.workflow_version)?;
        validate_parameters(definition, &workflow.parameters)?;
        validate_targets(app, definition, &workflow.targets)?;
    }
    Ok(())
}

fn validate_targets(
    app: &AppHandle,
    definition: &WorkflowDefinition,
    targets: &HashMap<String, SavedTargetBinding>,
) -> Result<(), String> {
    let mut required = HashSet::new();
    for step in definition.steps {
        if knowledge::actions::require(step.action)?
            .execution_capability
            .is_some()
        {
            required.insert(step.id);
        }
    }
    if targets.keys().map(String::as_str).collect::<HashSet<_>>() != required {
        return Err("Saved Workflow target bindings do not match its definition".into());
    }
    for step in definition.steps {
        let action = knowledge::actions::require(step.action)?;
        let Some(capability) = action.execution_capability else {
            continue;
        };
        let binding = targets
            .get(step.id)
            .ok_or_else(|| "Saved Workflow is missing an execution target".to_string())?;
        if binding.action_id != action.id {
            return Err("Saved Workflow action binding does not match its definition".into());
        }
        settings::resolve(
            app,
            binding.target.clone().with_temperature(0.2),
            capability,
        )
        .map_err(|error| error.to_string())?;
    }
    Ok(())
}

fn validate_parameters(definition: &WorkflowDefinition, value: &Value) -> Result<(), String> {
    let parameters = value
        .as_object()
        .ok_or("Saved Workflow parameters must be an object")?;
    if parameters
        .keys()
        .map(String::as_str)
        .collect::<HashSet<_>>()
        != definition
            .parameters
            .iter()
            .map(|parameter| parameter.key)
            .collect::<HashSet<_>>()
    {
        return Err("Saved Workflow parameters do not match its definition".into());
    }
    for parameter in definition.parameters {
        let value = parameters
            .get(parameter.key)
            .ok_or_else(|| "Saved Workflow parameter is missing".to_string())?;
        let valid = match parameter.kind {
            WorkflowParameterKind::Text => {
                value.as_str().is_some_and(|value| !value.trim().is_empty())
            }
            WorkflowParameterKind::Number => value.as_f64().is_some_and(|value| value.is_finite()),
        };
        if !valid {
            return Err(format!(
                "Saved Workflow parameter is invalid: {}",
                parameter.key
            ));
        }
    }
    Ok(())
}
