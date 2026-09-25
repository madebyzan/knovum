use std::path::Path;

use schemars::schema_for;
use ts_rs::{Config, TS};

pub fn export(project_root: &Path) -> Result<(), String> {
    let output = project_root.join("src/features/spaces/contracts/generated");
    let config = Config::new().with_out_dir(&output).with_large_int("number");
    export_types(&config)?;
    export_schemas(&output)
}

fn export_types(config: &Config) -> Result<(), String> {
    use crate::spaces::{assistant, authorization, runs, workflows};

    workflows::BuiltInWorkflowTemplateId::export_all(config).map_err(export_error)?;
    runs::RunSummary::export_all(config).map_err(export_error)?;
    runs::StepSummary::export_all(config).map_err(export_error)?;
    runs::RunEvent::export_all(config).map_err(export_error)?;
    authorization::ApprovalSummary::export_all(config).map_err(export_error)?;
    assistant::ConversationSummary::export_all(config).map_err(export_error)?;
    assistant::AssistantTaskAccepted::export_all(config).map_err(export_error)?;
    assistant::AssistantTaskUpdate::export_all(config).map_err(export_error)?;
    assistant::AssistantPermissionOption::export_all(config).map_err(export_error)?;
    assistant::AssistantMessage::export_all(config).map_err(export_error)?;
    Ok(())
}

fn export_schemas(output: &Path) -> Result<(), String> {
    std::fs::create_dir_all(output).map_err(io_error)?;
    let workflow = schema_for!(crate::spaces::workflows::WorkflowDefinition);
    write_json(&output.join("workflow-definition.schema.json"), &workflow)?;
    let action_schemas = crate::spaces::actions::definitions()
        .iter()
        .map(|definition| {
            (
                definition.id.to_owned(),
                crate::spaces::actions::parameter_schema(definition),
            )
        })
        .collect::<serde_json::Map<_, _>>();
    write_json(
        &output.join("space-action-parameters.schema.json"),
        &action_schemas,
    )
}

fn write_json(path: &Path, value: &impl serde::Serialize) -> Result<(), String> {
    let bytes = serde_json::to_vec_pretty(value).map_err(|error| error.to_string())?;
    std::fs::write(path, bytes).map_err(io_error)
}

fn export_error(error: ts_rs::ExportError) -> String {
    error.to_string()
}

fn io_error(error: std::io::Error) -> String {
    error.to_string()
}

#[cfg(test)]
mod tests;
