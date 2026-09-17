use std::collections::HashSet;

use super::contracts::{SaveWorkflowRunRequest, WorkflowStepStatus};

pub struct RunState<'a> {
    pub status: &'static str,
    pub error: &'a str,
}

pub fn validate(request: &SaveWorkflowRunRequest) -> Result<RunState<'_>, String> {
    validate_label(&request.id, "Workflow run ID")?;
    validate_label(&request.space_id, "Workflow run Space ID")?;
    validate_label(&request.workflow_id, "Workflow ID")?;
    let definition =
        super::super::workflows::require(&request.workflow_id, request.workflow_version)?;
    if request.steps.len() != definition.steps.len() {
        return Err("Creation Workflow steps do not match its registered definition".into());
    }
    let mut ids = HashSet::new();
    let mut running_steps = 0;
    let mut failed_steps = 0;
    for (step, expected) in request.steps.iter().zip(definition.steps) {
        validate_label(&step.id, "Workflow step ID")?;
        validate_label(&step.action, "Workflow step action")?;
        if !ids.insert(&step.id) {
            return Err("Creation Workflow step IDs must be unique".into());
        }
        if step.id != expected.id || step.action != expected.action {
            return Err("Creation Workflow step does not match its registered definition".into());
        }
        let failed = matches!(step.status, WorkflowStepStatus::Failed);
        failed_steps += usize::from(failed);
        running_steps += usize::from(matches!(step.status, WorkflowStepStatus::Running));
        if failed != !step.error.trim().is_empty() {
            return Err("Creation Workflow step error does not match its status".into());
        }
    }
    if running_steps > 1 {
        return Err("Creation Workflow cannot run multiple steps concurrently".into());
    }
    if running_steps > 0 && failed_steps > 0 {
        return Err("Creation Workflow cannot continue after a step fails".into());
    }
    validate_dependencies(request, definition)?;
    Ok(run_state(request))
}

pub fn next_step(request: &SaveWorkflowRunRequest) -> Result<Option<String>, String> {
    validate(request)?;
    if request
        .steps
        .iter()
        .any(|step| matches!(step.status, WorkflowStepStatus::Running))
    {
        return Err("Creation Workflow cannot plan while a step is running".into());
    }
    if request
        .steps
        .iter()
        .any(|step| matches!(step.status, WorkflowStepStatus::Failed))
    {
        return Ok(None);
    }
    let definition =
        super::super::workflows::require(&request.workflow_id, request.workflow_version)?;
    Ok(definition
        .steps
        .iter()
        .find(|expected| {
            let current = request
                .steps
                .iter()
                .find(|step| step.id == expected.id)
                .expect("registered workflow steps were validated");
            matches!(current.status, WorkflowStepStatus::Pending)
                && expected.depends_on.iter().all(|dependency| {
                    request.steps.iter().any(|step| {
                        step.id == *dependency
                            && matches!(step.status, WorkflowStepStatus::Succeeded)
                    })
                })
        })
        .map(|step| step.id.to_string()))
}

fn validate_dependencies(
    request: &SaveWorkflowRunRequest,
    definition: &crate::spaces::derivation::workflows::WorkflowDefinition,
) -> Result<(), String> {
    for (step, expected) in request.steps.iter().zip(definition.steps) {
        if matches!(
            step.status,
            WorkflowStepStatus::Running | WorkflowStepStatus::Succeeded
        ) && expected.depends_on.iter().any(|dependency| {
            request.steps.iter().any(|candidate| {
                candidate.id == *dependency
                    && !matches!(candidate.status, WorkflowStepStatus::Succeeded)
            })
        }) {
            return Err("Creation Workflow step started before its dependencies completed".into());
        }
    }
    Ok(())
}

fn run_state(request: &SaveWorkflowRunRequest) -> RunState<'_> {
    if request
        .steps
        .iter()
        .any(|step| matches!(step.status, WorkflowStepStatus::Running))
    {
        RunState {
            status: "running",
            error: "",
        }
    } else if let Some(failed) = request
        .steps
        .iter()
        .find(|step| matches!(step.status, WorkflowStepStatus::Failed))
    {
        RunState {
            status: "failed",
            error: failed.error.trim(),
        }
    } else if request
        .steps
        .iter()
        .all(|step| matches!(step.status, WorkflowStepStatus::Succeeded))
    {
        RunState {
            status: "completed",
            error: "",
        }
    } else {
        RunState {
            status: "running",
            error: "",
        }
    }
}

fn validate_label(value: &str, label: &str) -> Result<(), String> {
    if value.trim().is_empty() || value.len() > 256 {
        Err(format!("{label} is invalid"))
    } else {
        Ok(())
    }
}
