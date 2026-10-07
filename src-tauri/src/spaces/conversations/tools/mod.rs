use std::collections::{BTreeSet, HashMap};

use serde_json::{json, Value};
use sha2::{Digest, Sha256};

use crate::{
    ai::contracts::ModelToolDefinition,
    spaces::actions::{self, SpaceActionDefinition},
};

pub struct CatalogToolBinding {
    pub source: crate::extensions::contracts::AgentToolBinding,
    pub read_only: bool,
}

pub struct ToolCatalog {
    pub tools: Vec<ModelToolDefinition>,
    pub by_wire_name: HashMap<String, CatalogToolBinding>,
    pub digest: String,
}

pub fn project() -> Result<ToolCatalog, String> {
    let mut by_wire_name = HashMap::new();
    let mut names = BTreeSet::new();
    let mut tools = Vec::new();
    for definition in actions::definitions() {
        let wire_name = wire_name(definition);
        if !names.insert(wire_name.clone()) {
            return Err("tool-schema-invalid: deterministic tool name collision".into());
        }
        tools.push(ModelToolDefinition {
            name: wire_name.clone(),
            description: description(definition),
            input_schema: schema(definition),
        });
        by_wire_name.insert(
            wire_name,
            CatalogToolBinding {
                source: crate::extensions::contracts::AgentToolBinding::SpaceAction {
                    action_id: definition.id.into(),
                    action_version: definition.version,
                },
                read_only: definition.impact == actions::SpaceActionImpact::ReadOnly,
            },
        );
    }
    let encoded = serde_json::to_vec(&tools).map_err(|error| error.to_string())?;
    Ok(ToolCatalog {
        tools,
        by_wire_name,
        digest: hex::encode(Sha256::digest(encoded)),
    })
}

impl ToolCatalog {
    pub fn extend(
        &mut self,
        snapshot: &crate::extensions::contracts::AgentExtensionSnapshot,
    ) -> Result<(), String> {
        for tool in &snapshot.tools {
            if self.by_wire_name.contains_key(&tool.name) {
                return Err("agent-tool-wire-name-collision".into());
            }
            self.tools.push(ModelToolDefinition {
                name: tool.name.clone(),
                description: tool.description.clone(),
                input_schema: tool.input_schema.clone(),
            });
            self.by_wire_name.insert(
                tool.name.clone(),
                CatalogToolBinding {
                    source: tool.binding.clone(),
                    read_only: tool.read_only,
                },
            );
        }
        self.digest = hex::encode(Sha256::digest(
            serde_json::to_vec(&(&self.digest, &snapshot.digest, &self.tools))
                .map_err(|error| error.to_string())?,
        ));
        Ok(())
    }
    pub fn action(&self, name: &str) -> Result<&'static SpaceActionDefinition, String> {
        match &self
            .by_wire_name
            .get(name)
            .ok_or("agent-tool-not-selected")?
            .source
        {
            crate::extensions::contracts::AgentToolBinding::SpaceAction {
                action_id,
                action_version,
            } => {
                let definition = actions::require(action_id)?;
                if definition.version != *action_version {
                    return Err("space-action-version-changed".into());
                }
                Ok(definition)
            }
            _ => Err("agent-tool-is-not-space-action".into()),
        }
    }
}

fn wire_name(definition: &SpaceActionDefinition) -> String {
    format!(
        "knovum_{}_v{}",
        definition.id.replace('-', "_"),
        definition.version
    )
}

fn description(definition: &SpaceActionDefinition) -> String {
    match definition.id {
        "create-note" => "Create an editable note in the current Space.".into(),
        "answer-with-sources" => {
            "Answer a question using selected knowledge sources and return verified citations."
                .into()
        }
        "list-space-contents" => "List current canvas resources, including unindexed images, videos, files and notes. Paginate with nextOffset; this is the authoritative inventory, independent of search indexing.".into(),
        "search-knowledge" => {
            "Search indexed content in the current Space using keywords. Returns revision-aware excerpts and locators; empty matches do not mean the canvas is empty."
                .into()
        }
        "read-node" => "Read one exact current canvas node by ID, including its content and indexed passages. Images are delivered as actual visual input to vision-capable models, including current crop, rotation and adjustments. A local assetPath alone is not visual evidence; original asset files remain available after output workspace cleanup.".into(),
        "draft-document" => "Create an editable document from selected knowledge resources.".into(),
        "draft-script" => "Draft an editable video script from selected resources.".into(),
        "summarize-meeting" => "Create an editable summary from a meeting transcript.".into(),
        "draft-social-post" => "Create editable social copy grounded in selected knowledge.".into(),
        "generate-visualization" => "Generate an image that visualizes selected knowledge.".into(),
        "generate-image" => "Generate an image from an editable document brief.".into(),
        "generate-video" => "Generate a video from a document or ordered image sequence.".into(),
        "add-video-subtitles" => "Add subtitles to one video using existing timed cues or local speech recognition and publish a new video. Does not create an intermediate subtitle resource or require a model account.".into(),
        "render-presentation" => "Render a presentation from an editable document outline.".into(),
        _ => format!("Execute Knovum Space Action {}.", definition.id),
    }
}

fn schema(definition: &SpaceActionDefinition) -> Value {
    let mut schema = actions::parameter_schema(definition);
    let object = schema
        .as_object_mut()
        .expect("generated Action schema must be an object");
    let properties = object.entry("properties").or_insert_with(|| json!({}));
    let properties = properties
        .as_object_mut()
        .expect("generated Action properties must be an object");
    if !definition.input_roles.is_empty() {
        properties.insert(
            "inputNodeIds".into(),
            json!({
                "type": "array", "items": {"type": "string", "minLength": 1},
                "minItems": 1, "maxItems": 64,
                "description": "Exact source resource IDs in primary-first order."
            }),
        );
        object
            .entry("required")
            .or_insert_with(|| json!([]))
            .as_array_mut()
            .expect("generated required must be an array")
            .push(Value::String("inputNodeIds".into()));
    }
    schema
}

#[cfg(test)]
mod tests;
