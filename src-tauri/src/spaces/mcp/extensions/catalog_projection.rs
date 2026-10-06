use super::super::SpaceMcpServer;
use crate::extensions::contracts::{AgentExtensionTool, AgentToolBinding};
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

impl SpaceMcpServer {
    pub(in crate::spaces::mcp) fn selected_resources(&self) -> Vec<ResourceBinding<'_>> {
        let Some(snapshot) = &self.conversation_extensions else {
            return Vec::new();
        };
        let mut resources = Vec::new();
        for tool in &snapshot.tools {
            let AgentToolBinding::McpReadResource { connection_id } = &tool.binding else {
                continue;
            };
            let uris = tool.input_schema["properties"]["uri"]["enum"]
                .as_array()
                .expect("projected resource enum");
            let mut declarations = snapshot.catalogs[connection_id].resources.clone();
            for uri in uris {
                if !declarations
                    .iter()
                    .any(|resource| uri.as_str() == Some(&resource.uri))
                {
                    let uri = uri.as_str().expect("projected resource URI");
                    declarations.push(Resource::new(uri, uri));
                }
            }
            for value in &declarations {
                if !uris.iter().any(|uri| uri.as_str() == Some(&value.uri)) {
                    continue;
                }
                let mut declaration = value.clone();
                let uri = declaration.uri.clone();
                declaration.uri = format!("knovum-mcp-resource://{connection_id}/{}", digest(&uri));
                resources.push(ResourceBinding {
                    declaration,
                    uri,
                    tool,
                });
            }
        }
        resources
    }
    pub(in crate::spaces::mcp) fn selected_prompts(&self) -> Vec<PromptBinding<'_>> {
        let Some(snapshot) = &self.conversation_extensions else {
            return Vec::new();
        };
        let mut prompts = Vec::new();
        for tool in &snapshot.tools {
            let AgentToolBinding::McpGetPrompt { connection_id } = &tool.binding else {
                continue;
            };
            let names: Vec<_> = tool.input_schema["oneOf"]
                .as_array()
                .expect("projected prompt alternatives")
                .iter()
                .map(|variant| &variant["properties"]["name"]["const"])
                .collect();
            for value in &snapshot.catalogs[connection_id].prompts {
                if !names.iter().any(|name| name.as_str() == Some(&value.name)) {
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
}
fn digest(value: &str) -> String {
    use sha2::{Digest, Sha256};
    format!("{:x}", Sha256::digest(value.as_bytes()))
}
