use super::super::contracts::{
    AgentConnectionSelection, AgentExtensionSelection, ConnectorDefinition, McpConnection,
    PluginInstallation, SkillRevision,
};
use std::collections::BTreeSet;

pub(crate) fn resolve(
    requested: &AgentExtensionSelection,
    plugins: &[PluginInstallation],
    skills: &[SkillRevision],
    connectors: &[ConnectorDefinition],
    connections: &[McpConnection],
) -> Result<(AgentExtensionSelection, Vec<PluginInstallation>), String> {
    let mut resolved = requested.clone();
    let mut selected_plugins = Vec::new();
    let mut ids = BTreeSet::new();
    for id in &requested.skill_ids {
        let skill = skills
            .iter()
            .find(|skill| &skill.id == id)
            .ok_or("skill-not-found")?;
        if skill.plugin_id.is_some() {
            return Err("plugin-components-require-package-selection".into());
        }
    }
    for choice in &requested.connections {
        let connection = connections
            .iter()
            .find(|connection| connection.id == choice.connection_id)
            .ok_or("mcp-connection-not-found")?;
        let connector = connectors
            .iter()
            .find(|connector| connector.id == connection.connector_id)
            .ok_or("connector-definition-not-found")?;
        if connector.plugin_id.is_some() {
            return Err("plugin-components-require-package-selection".into());
        }
    }
    for id in &requested.plugin_ids {
        if !ids.insert(id) {
            return Err("selected-plugin-duplicated".into());
        }
        let plugin = plugins
            .iter()
            .find(|plugin| &plugin.id == id)
            .ok_or("plugin-not-found")?;
        require_package(plugin, skills, connectors, connections)?;
        selected_plugins.push(plugin.clone());
        resolved.skill_ids.extend(
            skills
                .iter()
                .filter(|skill| skill.plugin_id.as_ref() == Some(id))
                .map(|skill| skill.id.clone()),
        );
        for connector in connectors
            .iter()
            .filter(|connector| connector.plugin_id.as_ref() == Some(id))
        {
            let connection = connections
                .iter()
                .find(|connection| connection.connector_id == connector.id)
                .ok_or("plugin-connection-missing")?;
            resolved.connections.push(AgentConnectionSelection {
                connection_id: connection.id.clone(),
                tool_names: Vec::new(),
                resource_uris: Vec::new(),
                resource_templates: Vec::new(),
                prompt_names: Vec::new(),
            });
        }
    }
    Ok((resolved, selected_plugins))
}

pub(crate) fn require_package(
    plugin: &PluginInstallation,
    skills: &[SkillRevision],
    connectors: &[ConnectorDefinition],
    connections: &[McpConnection],
) -> Result<(), String> {
    if !plugin.enabled {
        return Err("selected-plugin-disabled".into());
    }
    if plugin.revision.components.iter().any(|component| {
        !component.supported && matches!(component.kind.as_str(), "skill" | "mcp" | "connector")
    }) {
        return Err("plugin-requires-unsupported-capabilities".into());
    }
    if skills
        .iter()
        .filter(|skill| skill.plugin_id.as_ref() == Some(&plugin.id))
        .any(|skill| !skill.enabled || !skill.unsupported_requirements.is_empty())
    {
        return Err("plugin-skill-unavailable".into());
    }
    for component in plugin
        .revision
        .components
        .iter()
        .filter(|component| component.kind == "skill")
    {
        if !skills.iter().any(|skill| {
            skill.plugin_id.as_ref() == Some(&plugin.id) && skill.name == component.name
        }) {
            return Err("plugin-skill-missing".into());
        }
    }
    for skill in skills
        .iter()
        .filter(|skill| skill.plugin_id.as_ref() == Some(&plugin.id))
    {
        for requirement in &skill.requirements {
            if !plugin
                .revision
                .connectors
                .iter()
                .any(|declaration| &declaration.component_name == requirement)
            {
                return Err("plugin-skill-requires-undeclared-connector".into());
            }
        }
    }
    for declaration in &plugin.revision.connectors {
        let connector = connectors
            .iter()
            .find(|connector| {
                connector.plugin_id.as_ref() == Some(&plugin.id)
                    && connector.component_name.as_ref() == Some(&declaration.component_name)
            })
            .ok_or("plugin-connector-missing")?;
        connector.configured_endpoint()?;
        let mut bindings = connections
            .iter()
            .filter(|connection| connection.connector_id == connector.id);
        if bindings.next().is_none() {
            return Err("plugin-connection-missing".into());
        }
        if bindings.next().is_some() {
            return Err("plugin-connection-binding-ambiguous".into());
        }
    }
    Ok(())
}
