use super::{contracts::AgentExtensionAvailability, database};
use tauri::AppHandle;

#[tauri::command]
pub async fn get_agent_extension_availability(
    app: AppHandle,
) -> Result<AgentExtensionAvailability, String> {
    super::plugins::ensure_builtin(&app).await?;
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
    let skills: Vec<_> = super::skills::list(&pool)
        .await?
        .into_iter()
        .filter(|skill| {
            skill.enabled
                && skill.plugin_id.as_ref().is_none_or(|id| {
                    plugins
                        .iter()
                        .any(|plugin| &plugin.id == id && plugin.enabled)
                })
        })
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
        .collect();
    let default_selection = super::contracts::AgentExtensionSelection {
        skill_ids: skills
            .iter()
            .filter(|skill| {
                skill.plugin_id.as_ref().is_some_and(|id| {
                    plugins
                        .iter()
                        .any(|plugin| &plugin.id == id && plugin.builtin)
                })
            })
            .map(|skill| skill.id.clone())
            .collect(),
        connections: Vec::new(),
    };
    Ok(AgentExtensionAvailability {
        default_selection,
        skills,
        connectors,
        connections,
        diagnostics,
    })
}
