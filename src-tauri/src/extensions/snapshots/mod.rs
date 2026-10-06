mod skill_bindings;
mod tools;
use super::{
    content,
    contracts::{
        AgentExtensionSelection, AgentExtensionSnapshot, AgentSkillFile, SkillFileEncoding,
    },
    database, ExtensionState,
};
use base64::Engine;
use std::{collections::BTreeSet, sync::Arc};
use tauri::{AppHandle, Manager};

pub(crate) async fn prepare(
    app: &AppHandle,
    selection: &AgentExtensionSelection,
    read_only: bool,
    local_tools: &[String],
    interaction: Option<Arc<dyn super::mcp::McpInteraction>>,
) -> Result<AgentExtensionSnapshot, String> {
    super::plugins::ensure_builtin(app).await?;
    let state = app.state::<ExtensionState>();
    let _guard = state.content_lock.lock().await;
    let pool = database::pool(app).await?;
    let mut snapshot = AgentExtensionSnapshot {
        id: uuid::Uuid::new_v4().to_string(),
        digest: String::new(),
        skills: Vec::new(),
        files: Vec::new(),
        skill_tool_bindings: Default::default(),
        connectors: Vec::new(),
        connections: Vec::new(),
        accounts: Vec::new(),
        catalogs: Default::default(),
        tools: Vec::new(),
    };
    let mut names = BTreeSet::new();
    let mut retained = BTreeSet::new();
    for id in &selection.skill_ids {
        let skill = super::skills::load(&pool, id).await?;
        if !skill.unsupported_requirements.is_empty() {
            return Err(format!(
                "skill-requires-unsupported-capabilities: {}: {:?}",
                skill.name, skill.unsupported_requirements
            ));
        }
        if !skill.enabled {
            return Err("selected-skill-disabled".into());
        }
        if let Some(plugin_id) = &skill.plugin_id {
            if !super::plugins::load(&pool, plugin_id).await?.enabled {
                return Err("selected-skill-plugin-disabled".into());
            }
        }
        if !names.insert(skill.name.clone()) {
            return Err(format!("selected-skill-name-conflict: {}", skill.name));
        }
        let files = super::skills::files(app, &skill).await?;
        for (path, bytes) in files {
            let (encoding, encoded) = match std::str::from_utf8(&bytes) {
                Ok(text) => (SkillFileEncoding::Utf8, text.to_owned()),
                Err(_) => (
                    SkillFileEncoding::Base64,
                    base64::engine::general_purpose::STANDARD.encode(&bytes),
                ),
            };
            snapshot.files.push(AgentSkillFile {
                path: format!("/skills/{}/{path}", skill.name),
                encoding,
                content: encoded,
                digest: content::digest(&bytes),
            });
        }
        let lease = if let Some(plugin_id) = &skill.plugin_id {
            let plugin = super::plugins::load(&pool, plugin_id).await?;
            format!("plugins/{plugin_id}/revisions/{}", plugin.revision.digest)
        } else {
            format!("skills/{}/revisions/{}", skill.id, skill.digest)
        };
        retained.insert(lease);
        snapshot.skills.push(skill);
    }
    if serde_json::to_vec(&snapshot.files)
        .map_err(|error| error.to_string())?
        .len()
        > 8 * 1024 * 1024
    {
        return Err("selected-skills-exceed-8-mib-snapshot-budget".into());
    }
    let mut selected_connections = BTreeSet::new();
    for choices in &selection.connections {
        let id = &choices.connection_id;
        if !selected_connections.insert(id) {
            return Err("selected-mcp-connection-duplicated".into());
        }
        let connection = super::mcp::connections::load(&pool, id).await?;
        super::mcp::connections::require_current(app, &connection).await?;
        let connector =
            super::connectors::definitions::load(&pool, &connection.connector_id).await?;
        if let Some(plugin_id) = &connector.plugin_id {
            let plugin = super::plugins::load(&pool, plugin_id).await?;
            retained.insert(format!(
                "plugins/{plugin_id}/revisions/{}",
                plugin.revision.digest
            ));
        }
        if let Some(account_id) = &connection.account_id {
            if !snapshot
                .accounts
                .iter()
                .any(|account| &account.id == account_id)
            {
                snapshot
                    .accounts
                    .push(super::connectors::accounts::load(&pool, account_id).await?);
            }
        }
        if !snapshot
            .connectors
            .iter()
            .any(|definition| definition.id == connector.id)
        {
            snapshot.connectors.push(connector);
        }
        snapshot.connections.push(connection);
    }
    for skill in &snapshot.skills {
        for name in &skill.requirements {
            if !snapshot.connections.iter().any(|connection| {
                snapshot.connectors.iter().any(|connector| {
                    connector.id == connection.connector_id
                        && connector.plugin_id == skill.plugin_id
                        && connector.component_name.as_ref() == Some(name)
                })
            }) {
                return Err(format!(
                    "selected-skill-requires-connection: {}: {name}",
                    skill.name
                ));
            }
        }
    }
    sqlx::query("INSERT INTO content_leases(snapshot_id,document) VALUES($1,$2)")
        .bind(&snapshot.id)
        .bind(database::encode(&retained)?)
        .execute(&pool)
        .await
        .map_err(|error| error.to_string())?;
    drop(_guard);
    let result = async {
        for (connection, choices) in snapshot.connections.iter().zip(&selection.connections) {
            let catalog =
                super::mcp::sessions::inspect(app, &snapshot, connection, interaction.clone())
                    .await?;
            snapshot.tools.extend(tools::project(
                connection,
                catalog.clone(),
                read_only,
                choices,
            )?);
            snapshot.catalogs.insert(connection.id.clone(), catalog);
        }
        snapshot.skill_tool_bindings = skill_bindings::resolve(&snapshot, local_tools)?;
        snapshot.digest = content::digest(
            &serde_json::to_vec(&(
                &snapshot.skills,
                &snapshot.files,
                &snapshot.skill_tool_bindings,
                &snapshot.connectors,
                &snapshot.connections,
                &snapshot.accounts,
                &snapshot.catalogs,
                &snapshot.tools,
                "authorization-v1",
            ))
            .map_err(|error| error.to_string())?,
        );
        require_current(app, &snapshot).await?;
        Ok(snapshot.clone())
    }
    .await;
    super::mcp::sessions::release(app, &snapshot.id).await;
    if result.is_err() {
        super::reclamation::release(app, &snapshot.id).await?;
    }
    result
}

pub(crate) async fn require_current(
    app: &AppHandle,
    snapshot: &AgentExtensionSnapshot,
) -> Result<(), String> {
    let pool = database::pool(app).await?;
    for account in &snapshot.accounts {
        super::connectors::accounts::require_current(&pool, account).await?;
    }
    for frozen in &snapshot.skills {
        let current = super::skills::load(&pool, &frozen.id).await?;
        if !current.enabled || current.digest != frozen.digest {
            return Err("skill-revoked-or-revised".into());
        }
        if let Some(plugin_id) = &current.plugin_id {
            if !super::plugins::load(&pool, plugin_id).await?.enabled {
                return Err("skill-plugin-disabled".into());
            }
        }
    }
    for connector in &snapshot.connectors {
        super::connectors::definitions::require_current(&pool, connector).await?;
    }
    for connection in &snapshot.connections {
        super::mcp::connections::require_current(app, connection).await?;
    }
    Ok(())
}

#[cfg(test)]
#[path = "tests/tools.rs"]
mod tools_tests;
