use super::{contracts::AgentExtensionAvailability, database};
use tauri::AppHandle;

#[tauri::command]
pub async fn get_agent_extension_availability(
    app: AppHandle,
) -> Result<AgentExtensionAvailability, String> {
    super::plugins::reconcile_bundled_plugins(&app).await?;
    let pool = database::pool(&app).await?;
    let plugins = super::plugins::list(&pool).await?;
    let connectors = super::connectors::definitions::list(&pool).await?;
    let candidates = super::mcp::connections::list(&pool)
        .await?
        .into_iter()
        .filter(|connection| connection.enabled)
        .collect::<Vec<_>>();
    let mut diagnostics = Vec::new();
    let mut connections = Vec::new();
    for connection in candidates {
        match super::mcp::connections::require_current(&app, &connection).await {
            Ok(()) => connections.push(connection),
            Err(error) => diagnostics.push(format!("{}: {error}", connection.name)),
        }
    }
    let all_skills = super::skills::list(&pool).await?;
    let skills: Vec<_> = all_skills
        .iter()
        .filter(|skill| skill.enabled)
        .filter(|skill| {
            if !skill.unsupported_requirements.is_empty() {
                diagnostics.push(format!(
                    "{}: unsupported requirements: {:?}",
                    skill.name, skill.unsupported_requirements
                ));
                return false;
            }
            let missing = skill
                .requirements
                .iter()
                .filter(|name| {
                    !connections.iter().any(|connection| {
                        connectors.iter().any(|connector| {
                            connector.id == connection.connector_id
                                && connector.plugin_id == skill.plugin_id
                                && connector.component_name.as_ref() == Some(name)
                        })
                    })
                })
                .collect::<Vec<_>>();
            if !missing.is_empty() {
                diagnostics.push(format!(
                    "{}: required MCP connections are not enabled: {:?}",
                    skill.name, missing
                ));
            }
            missing.is_empty()
        })
        .cloned()
        .collect();
    let available_plugins: Vec<_> = plugins
        .into_iter()
        .filter(|plugin| {
            if !plugin.enabled {
                return false;
            }
            match super::snapshots::selection::require_package(
                plugin,
                &all_skills,
                &connectors,
                &connections,
            ) {
                Ok(()) => true,
                Err(error) => {
                    diagnostics.push(format!("{}: {error}", plugin.revision.manifest.name));
                    false
                }
            }
        })
        .collect();
    let default_selection = super::contracts::AgentExtensionSelection {
        plugin_ids: available_plugins
            .iter()
            .filter(|plugin| plugin.builtin)
            .map(|plugin| plugin.id.clone())
            .collect(),
        skill_ids: Vec::new(),
        connections: Vec::new(),
    };
    let personal_skills = skills
        .into_iter()
        .filter(|skill| skill.plugin_id.is_none())
        .collect();
    let standalone_connections = connections
        .into_iter()
        .filter(|connection| {
            connectors.iter().any(|connector| {
                connector.id == connection.connector_id && connector.plugin_id.is_none()
            })
        })
        .collect();
    Ok(AgentExtensionAvailability {
        plugins: available_plugins,
        default_selection,
        skills: personal_skills,
        connectors,
        connections: standalone_connections,
        diagnostics,
    })
}
