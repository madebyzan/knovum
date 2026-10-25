use serde::{Serialize, Serializer};
use serde_json::Value;

use crate::ai::contracts::{ExecutionCapability, RoutingPurpose};
use crate::foundation::app_error::AppError;

pub(crate) mod canvas;
mod canvas_inputs;
pub(crate) mod catalog;
pub(crate) mod input_selection;
mod invocation;

#[cfg(test)]
mod tests;

#[derive(Clone, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub struct SpaceActionDefinition {
    pub id: &'static str,
    pub version: u16,
    pub impact: SpaceActionImpact,
    pub execution_mode: SpaceActionExecutionMode,
    pub input_consistency: SpaceActionInputConsistency,
    pub execution_capability: Option<ExecutionCapability>,
    pub routing_purpose: Option<RoutingPurpose>,
    pub input_roles: &'static [SpaceActionInputRole],
    /// How many input resources a Run needs. A generation reads selected resources as references
    /// when there are any and works from its prompt alone when there are none.
    pub min_inputs: usize,
    pub collection_input: Option<SpaceActionCollectionInput>,
    #[serde(serialize_with = "serialize_parameter_schema")]
    #[ts(type = "Record<string, unknown>")]
    pub parameters_schema: ParameterSchemaFactory,
    pub output_contract: SpaceActionOutputContract,
}

#[derive(Clone, Copy, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub struct SpaceActionCollectionInput {
    pub kind: &'static str,
    pub role: &'static str,
    pub max_items: usize,
}

#[derive(Clone, Copy, Eq, PartialEq, Serialize, schemars::JsonSchema, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub enum SpaceActionImpact {
    ReadOnly,
    CreatesResource,
    MutatesResource,
}

#[derive(Clone, Copy, Serialize, schemars::JsonSchema, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub enum SpaceActionExecutionMode {
    Immediate,
    Durable,
}

#[derive(Clone, Copy, Serialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub enum SpaceActionInputConsistency {
    PinnedRevision,
}

pub type ParameterSchemaFactory = fn() -> &'static Value;

#[derive(Clone, Copy, Eq, PartialEq, Serialize, schemars::JsonSchema, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub enum SpaceActionInputRole {
    Image,
    Document,
    Knowledge,
    Transcript,
    Video,
}

#[derive(Clone, Copy, Eq, PartialEq, Serialize, schemars::JsonSchema, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub enum SpaceActionOutputContract {
    Answer,
    Document,
    Image,
    KnowledgeChangeReceipt,
    KnowledgeNode,
    ResourceReceipt,
    SearchResults,
    SpaceEditReceipt,
    SpaceContents,
    Video,
}

pub fn definitions() -> &'static [SpaceActionDefinition] {
    catalog::ACTIONS
}

#[tauri::command]
pub fn list_space_actions() -> Vec<SpaceActionDefinition> {
    definitions().to_vec()
}

pub fn require(action_id: &str) -> Result<&'static SpaceActionDefinition, String> {
    catalog::ACTIONS
        .iter()
        .find(|definition| definition.id == action_id)
        .ok_or_else(|| format!("SpaceAction is not registered: {action_id}"))
}

pub fn validate_parameters(
    definition: &SpaceActionDefinition,
    value: &Value,
) -> Result<(), String> {
    let schema = parameter_schema(definition);
    let validator = jsonschema::validator_for(&schema)
        .map_err(|error| format!("SpaceAction schema is invalid: {error}"))?;
    if let Err(error) = validator.validate(value) {
        return Err(format!("SpaceAction parameters are invalid: {error}"));
    }
    if is_image_transform(definition.id) {
        let parameters: crate::spaces::derivation::image_operations::contracts::ImageTransformParameters = serde_json::from_value(value.clone()).map_err(crate::spaces::db::json_error)?;
        if parameters.title.trim().is_empty() {
            return Err("Image output title must not be blank".into());
        }
        parameters.operation.validate()?;
        parameters.execution.validate(&parameters.operation)?;
        if parameters.operation.action_kind() != definition.id {
            return Err("Image operation does not match its Action identity".into());
        }
    }
    Ok(())
}

pub fn parameter_schema(definition: &SpaceActionDefinition) -> Value {
    (definition.parameters_schema)().clone()
}

fn serialize_parameter_schema<S>(
    factory: &ParameterSchemaFactory,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    factory().serialize(serializer)
}

pub(crate) use invocation::{
    invoke, invoke_in_transaction, ActionInvocationOutcome, ActionInvocationRequest,
};

pub(crate) async fn invoke_with_app(
    app: &tauri::AppHandle,
    mut request: ActionInvocationRequest,
) -> Result<ActionInvocationOutcome, AppError> {
    resolve_target(app, &mut request)?;
    invoke(crate::spaces::db::pool(app).await?, request).await
}

fn resolve_target(
    app: &tauri::AppHandle,
    request: &mut ActionInvocationRequest,
) -> Result<(), AppError> {
    let definition = require(&request.action_id)?;
    if let Some(purpose) = definition.routing_purpose {
        let inputs = request
            .input_snapshot
            .as_object_mut()
            .ok_or("action-input-invalid: inputSnapshot must be an object")?;
        let targets = inputs
            .entry("targets")
            .or_insert_with(|| serde_json::json!({}))
            .as_object_mut()
            .ok_or("action-input-invalid: targets must be an object")?;
        let target = match targets.get(definition.id) {
            Some(value) => {
                serde_json::from_value(value.clone()).map_err(crate::spaces::db::json_error)?
            }
            None => crate::ai::settings::default_selection(app, purpose)?,
        };
        crate::ai::settings::resolve_task(
            app,
            target.clone(),
            definition
                .execution_capability
                .ok_or("action-target-invalid: execution capability missing")?,
        )?;
        targets.insert(
            definition.id.into(),
            serde_json::to_value(target).map_err(crate::spaces::db::json_error)?,
        );
    }
    Ok(())
}

pub(crate) fn is_image_transform(action_id: &str) -> bool {
    matches!(
        action_id,
        "upscale-image"
            | "generate-image-angle"
            | "remove-image-object"
            | "remove-image-background"
            | "extract-image-selection"
            | "generate-image-depth"
    )
}
