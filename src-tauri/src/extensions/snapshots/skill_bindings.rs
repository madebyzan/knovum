use super::super::contracts::{AgentExtensionSnapshot, AgentToolBinding};
use std::collections::BTreeMap;

pub(super) fn resolve(
    snapshot: &AgentExtensionSnapshot,
    local_tools: &[String],
) -> Result<BTreeMap<String, Vec<String>>, String> {
    let mut bindings = BTreeMap::new();
    for skill in &snapshot.skills {
        for include in &skill.included_tools {
            let (component, tool_name) = include
                .split_once('/')
                .map(|(component, name)| (component, Some(name)))
                .unwrap_or((include, None));
            let mut names = if local_tools.contains(include) {
                vec![include.clone()]
            } else {
                Vec::new()
            };
            for tool in &snapshot.tools {
                let Some((connection_id, actual)) = mcp_identity(&tool.binding) else {
                    continue;
                };
                let connection = snapshot
                    .connections
                    .iter()
                    .find(|connection| connection.id == connection_id)
                    .expect("projected connection");
                let connector = snapshot
                    .connectors
                    .iter()
                    .find(|connector| connector.id == connection.connector_id)
                    .expect("snapshot connector definition");
                if connector.plugin_id == skill.plugin_id
                    && connector.component_name.as_deref() == Some(component)
                    && tool_name.is_none_or(|name| name == actual)
                {
                    names.push(tool.name.clone());
                }
                if skill.plugin_id.is_none() && &tool.name == include {
                    names.push(tool.name.clone());
                }
            }
            names.sort();
            names.dedup();
            if names.is_empty() {
                return Err(format!(
                    "skill-tool-binding-unavailable: {}: {include}",
                    skill.name
                ));
            }
            if bindings
                .get(include)
                .is_some_and(|previous| previous != &names)
            {
                return Err(format!("skill-tool-binding-ambiguous: {include}"));
            }
            bindings.insert(include.clone(), names);
        }
    }
    Ok(bindings)
}

fn mcp_identity(binding: &AgentToolBinding) -> Option<(&str, &str)> {
    match binding {
        AgentToolBinding::McpTool {
            connection_id,
            tool_name,
            ..
        } => Some((connection_id, tool_name)),
        AgentToolBinding::McpReadResource { connection_id, .. } => {
            Some((connection_id, "resources/read"))
        }
        AgentToolBinding::McpGetPrompt { connection_id, .. } => {
            Some((connection_id, "prompts/get"))
        }
        AgentToolBinding::SpaceAction { .. } => None,
    }
}
