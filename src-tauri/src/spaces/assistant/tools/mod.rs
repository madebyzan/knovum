use std::collections::{BTreeSet, HashMap};

use serde_json::{json, Value};
use sha2::{Digest, Sha256};

use super::AssistantMode;
use crate::{
    ai::contracts::ModelToolDefinition,
    spaces::actions::{self, SpaceActionDefinition},
};

pub struct ToolCatalog {
    pub tools: Vec<ModelToolDefinition>,
    pub by_wire_name: HashMap<String, &'static SpaceActionDefinition>,
    pub digest: String,
}

pub fn project(mode: AssistantMode) -> Result<ToolCatalog, String> {
    let definitions = actions::definitions()
        .iter()
        .filter(|definition| match mode {
            AssistantMode::Ask | AssistantMode::Plan => {
                definition.impact == actions::SpaceActionImpact::ReadOnly
            }
            AssistantMode::Execute => true,
        })
        .collect::<Vec<_>>();
    let mut by_wire_name = HashMap::new();
    let mut names = BTreeSet::new();
    let mut tools = Vec::new();
    for definition in definitions {
        let wire_name = wire_name(definition);
        if !names.insert(wire_name.clone()) {
            return Err("tool-schema-invalid: deterministic tool name collision".into());
        }
        tools.push(ModelToolDefinition {
            name: wire_name.clone(),
            description: description(definition),
            input_schema: schema(definition),
        });
        by_wire_name.insert(wire_name, definition);
    }
    let encoded = serde_json::to_vec(&(mode, &tools)).map_err(|error| error.to_string())?;
    Ok(ToolCatalog {
        tools,
        by_wire_name,
        digest: format!("{:x}", Sha256::digest(encoded)),
    })
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
