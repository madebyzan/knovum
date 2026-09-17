use std::collections::HashSet;

use serde::Serialize;
use serde_json::Value;

use crate::ai::contracts::{ExecutionCapability, RoutingPurpose};

mod catalog;

#[cfg(test)]
mod tests;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeActionDefinition {
    pub id: &'static str,
    pub version: u16,
    pub execution_capability: Option<ExecutionCapability>,
    pub routing_purpose: Option<RoutingPurpose>,
    pub input_roles: &'static [&'static str],
    pub parameters: &'static [KnowledgeActionParameter],
    pub output_contract: &'static str,
}

#[derive(Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum KnowledgeActionParameterKind {
    Text,
    Number,
}

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeActionParameter {
    pub key: &'static str,
    pub kind: KnowledgeActionParameterKind,
}

pub fn definitions() -> &'static [KnowledgeActionDefinition] {
    catalog::ACTIONS
}

#[tauri::command]
pub fn list_knowledge_actions() -> Vec<KnowledgeActionDefinition> {
    definitions().to_vec()
}

pub fn require(id: &str) -> Result<&'static KnowledgeActionDefinition, String> {
    catalog::ACTIONS
        .iter()
        .find(|definition| definition.id == id)
        .ok_or_else(|| format!("Knowledge Action is not registered: {id}"))
}

pub fn validate_parameters(
    definition: &KnowledgeActionDefinition,
    value: &Value,
) -> Result<(), String> {
    let values = value
        .as_object()
        .ok_or("Knowledge Action parameters must be an object")?;
    if values.keys().map(String::as_str).collect::<HashSet<_>>()
        != definition
            .parameters
            .iter()
            .map(|parameter| parameter.key)
            .collect::<HashSet<_>>()
    {
        return Err(format!(
            "Knowledge Action parameters do not match: {}",
            definition.id
        ));
    }

    for parameter in definition.parameters {
        let value = &values[parameter.key];
        let valid = match parameter.kind {
            KnowledgeActionParameterKind::Text => {
                value.as_str().is_some_and(|text| !text.trim().is_empty())
            }
            KnowledgeActionParameterKind::Number => {
                value.as_f64().is_some_and(|number| number.is_finite())
            }
        };
        if !valid {
            return Err(format!(
                "Knowledge Action parameter is invalid: {}",
                parameter.key
            ));
        }
    }
    Ok(())
}
