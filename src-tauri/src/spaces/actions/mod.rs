use serde::{Serialize, Serializer};
use serde_json::Value;

use crate::ai::contracts::{ExecutionCapability, RoutingPurpose};

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

#[derive(Clone, Copy, Eq, PartialEq, Serialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub enum SpaceActionImpact {
    ReadOnly,
    CreatesResource,
    MutatesResource,
}

#[derive(Clone, Copy, Serialize, ts_rs::TS)]
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

#[derive(Clone, Copy, Eq, PartialEq, Serialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub enum SpaceActionInputRole {
    Document,
    Knowledge,
    Subtitle,
    Transcript,
    Video,
}

#[derive(Clone, Copy, Eq, PartialEq, Serialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub enum SpaceActionOutputContract {
    Answer,
    Document,
    Image,
    KnowledgeChangeReceipt,
    KnowledgeNode,
    Presentation,
    ResourceReceipt,
    SearchResults,
    SpaceEditReceipt,
    SpaceContents,
    Subtitle,
    Video,
}

impl SpaceActionOutputContract {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Answer => "answer",
            Self::Document => "document",
            Self::Image => "image",
            Self::KnowledgeChangeReceipt => "knowledge_change_receipt",
            Self::KnowledgeNode => "knowledge_node",
            Self::Presentation => "presentation",
            Self::ResourceReceipt => "resource_receipt",
            Self::SearchResults => "search_results",
            Self::SpaceEditReceipt => "space_edit_receipt",
            Self::SpaceContents => "space_contents",
            Self::Subtitle => "subtitle",
            Self::Video => "video",
        }
    }
}

pub fn definitions() -> &'static [SpaceActionDefinition] {
    catalog::ACTIONS
}

#[tauri::command]
pub fn list_space_actions() -> Vec<SpaceActionDefinition> {
    definitions().to_vec()
}

pub fn require(id: &str) -> Result<&'static SpaceActionDefinition, String> {
    catalog::ACTIONS
        .iter()
        .find(|definition| definition.id == id)
        .ok_or_else(|| format!("SpaceAction is not registered: {id}"))
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

pub(crate) use invocation::{invoke, ActionInvocationOutcome, ActionInvocationRequest};

pub(crate) async fn invoke_with_app(
    app: &tauri::AppHandle,
    request: ActionInvocationRequest,
) -> Result<ActionInvocationOutcome, String> {
    invoke(crate::spaces::db::pool(app).await?, request).await
}
