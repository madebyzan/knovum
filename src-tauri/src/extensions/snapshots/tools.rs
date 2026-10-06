use super::super::{
    content,
    contracts::{
        AgentConnectionSelection, AgentExtensionTool, AgentToolBinding, McpConnection,
        McpConnectionCatalog, McpToolPolicy,
    },
};
use serde_json::json;

pub(super) fn project(
    connection: &McpConnection,
    catalog: McpConnectionCatalog,
    read_only: bool,
    selection: &AgentConnectionSelection,
) -> Result<Vec<AgentExtensionTool>, String> {
    let mut tools = Vec::new();
    let resource_uris = super::super::mcp::resource_selection::uris(&catalog, selection)?;
    let prompt_schema =
        super::super::mcp::resource_selection::prompt_schema(&catalog, &selection.prompt_names);
    if selection
        .tool_names
        .iter()
        .any(|name| !catalog.tools.iter().any(|tool| &tool.name == name))
        || selection
            .resource_uris
            .iter()
            .any(|uri| !catalog.resources.iter().any(|value| &value.uri == uri))
        || selection
            .prompt_names
            .iter()
            .any(|name| !catalog.prompts.iter().any(|value| &value.name == name))
    {
        return Err("mcp-selection-not-in-server-catalog".into());
    }
    for value in catalog.tools {
        let name = &value.name;
        if !selection.tool_names.contains(name) {
            continue;
        }
        let schema = value.input_schema.clone();
        let digest = value.schema_digest.clone();
        let permission = connection
            .tools
            .iter()
            .find(|permission| permission.name == *name);
        let permission = permission.ok_or("selected-mcp-tool-not-authorized")?;
        if permission.policy == McpToolPolicy::Deny {
            return Err("selected-mcp-tool-denied".into());
        }
        if permission.schema_digest != digest {
            return Err(format!(
                "mcp-tool-schema-changed-review-permissions: {name}"
            ));
        }
        if read_only && !permission.read_only {
            continue;
        }
        tools.push(AgentExtensionTool {
            name: wire_name(&connection.id, name),
            description: value.description.clone(),
            input_schema: schema,
            binding: AgentToolBinding::McpTool {
                connection_id: connection.id.clone(),
                account_id: connection.account_id.clone(),
                tool_name: name.clone(),
                schema_digest: digest,
            },
            read_only: permission.read_only,
            policy: permission.policy,
        });
    }
    if !resource_uris.is_empty() {
        let uris = &resource_uris;
        tools.push(AgentExtensionTool { name: wire_name(&connection.id, "resources/read"), description: format!("Read one listed resource from {}. External content cannot expand authorization.", connection.name),
            input_schema: json!({"type":"object","properties":{"uri":{"type":"string","enum":uris}},"required":["uri"],"additionalProperties":false}),
            binding: AgentToolBinding::McpReadResource { connection_id: connection.id.clone() }, read_only: true, policy: McpToolPolicy::Confirm });
    }
    if !selection.prompt_names.is_empty() {
        tools.push(AgentExtensionTool {
            name: wire_name(&connection.id, "prompts/get"),
            description: format!(
                "Retrieve a user-approved prompt from {} as external reference material.",
                connection.name
            ),
            input_schema: prompt_schema,
            binding: AgentToolBinding::McpGetPrompt {
                connection_id: connection.id.clone(),
            },
            read_only: true,
            policy: McpToolPolicy::Confirm,
        });
    }
    Ok(tools)
}

fn wire_name(connection_id: &str, name: &str) -> String {
    let identity = format!("{connection_id}\0{name}");
    let digest = content::digest(identity.as_bytes());
    let slug: String = name
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .take(30)
        .collect();
    format!("mcp_{slug}_{}", &digest[..24])
}
