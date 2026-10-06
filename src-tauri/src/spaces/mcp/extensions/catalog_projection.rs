use crate::extensions::contracts::{AgentExtensionSnapshot, AgentExtensionTool, AgentToolBinding};
use rmcp::model::{Prompt, Resource};

pub(in crate::spaces::mcp) struct ResourceBinding<'a> {
    pub declaration: Resource,
    pub uri: String,
    pub tool: &'a AgentExtensionTool,
}
pub(in crate::spaces::mcp) struct PromptBinding<'a> {
    pub declaration: Prompt,
    pub name: String,
    pub tool: &'a AgentExtensionTool,
}

pub(in crate::spaces::mcp) fn selected_resources(
    snapshot: Option<&AgentExtensionSnapshot>,
) -> Vec<ResourceBinding<'_>> {
    let Some(snapshot) = snapshot else {
        return Vec::new();
    };
    let mut resources = Vec::new();
    for tool in &snapshot.tools {
        let AgentToolBinding::McpReadResource {
            connection_id,
            resource_uris,
        } = &tool.binding
        else {
            continue;
        };
        for uri in resource_uris {
            // Expanded URI templates have no static resource declaration in the catalog.
            let mut declaration = snapshot.catalogs[connection_id]
                .resources
                .iter()
                .find(|resource| &resource.uri == uri)
                .cloned()
                .unwrap_or_else(|| Resource::new(uri.clone(), uri.clone()));
            declaration.uri = format!("knovum-mcp-resource://{connection_id}/{}", digest(uri));
            resources.push(ResourceBinding {
                declaration,
                uri: uri.clone(),
                tool,
            });
        }
    }
    resources
}
pub(in crate::spaces::mcp) fn selected_prompts(
    snapshot: Option<&AgentExtensionSnapshot>,
) -> Vec<PromptBinding<'_>> {
    let Some(snapshot) = snapshot else {
        return Vec::new();
    };
    let mut prompts = Vec::new();
    for tool in &snapshot.tools {
        let AgentToolBinding::McpGetPrompt {
            connection_id,
            prompt_names,
        } = &tool.binding
        else {
            continue;
        };
        for value in &snapshot.catalogs[connection_id].prompts {
            if !prompt_names.contains(&value.name) {
                continue;
            }
            let mut declaration = value.clone();
            let name = declaration.name.clone();
            declaration.name = format!("prompt_{}", digest(&format!("{connection_id}/{name}")));
            prompts.push(PromptBinding {
                declaration,
                name,
                tool,
            });
        }
    }
    prompts
}
fn digest(value: &str) -> String {
    use sha2::{Digest, Sha256};
    format!("{:x}", Sha256::digest(value.as_bytes()))
}
