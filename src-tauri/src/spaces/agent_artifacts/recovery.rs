use std::collections::HashSet;

use sqlx::Row;
use tauri::{AppHandle, Manager};

use crate::ai::{
    capabilities, contracts,
    error::{AiError, AiResult},
};

#[tauri::command]
pub async fn list_recoverable_artifact_workspaces(
    app: AppHandle,
    space_id: String,
) -> AiResult<Vec<contracts::RecoverableAgentCreation>> {
    let root = capabilities::agent_artifacts::space_workspace(&app, &space_id)?;
    if !root.is_dir() {
        return Ok(Vec::new());
    }
    let mut entries = tokio::fs::read_dir(root)
        .await
        .map_err(|error| AiError::internal(error.to_string()))?;
    let mut recoverable = Vec::new();
    while let Some(entry) = entries
        .next_entry()
        .await
        .map_err(|error| AiError::internal(error.to_string()))?
    {
        let session_id = entry
            .file_name()
            .into_string()
            .map_err(|_| AiError::invalid("Agent workspace has an invalid session identifier"))?;
        uuid::Uuid::parse_str(&session_id)
            .map_err(|_| AiError::invalid("Agent workspace has an invalid session identifier"))?;
        let completed_at = super::super::derivation::workflow_runs::agent_runs::completed_at(
            &app,
            &space_id,
            &session_id,
        )
        .await
        .map_err(AiError::internal)?;
        let Some(completed_at) = completed_at else {
            continue;
        };
        let workspace = capabilities::agent_artifacts::workspace(&app, &space_id, &session_id)?;
        if !workspace.is_dir() {
            continue;
        }
        let artifacts = capabilities::agent_artifacts::collect(&workspace)?;
        if artifacts.is_empty() {
            continue;
        }
        recoverable.push((
            completed_at,
            contracts::RecoverableAgentCreation {
                session_id,
                artifacts,
            },
        ));
    }
    recoverable.sort_by_key(|entry| std::cmp::Reverse(entry.0));
    Ok(recoverable
        .into_iter()
        .map(|(_, session)| session)
        .collect())
}

pub async fn discard_orphaned_workspaces(app: &AppHandle) -> Result<(), String> {
    let root = app
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string())?
        .join("agent-workspaces")
        .join("spaces");
    if !tokio::fs::try_exists(&root)
        .await
        .map_err(|error| error.to_string())?
    {
        return Ok(());
    }
    let pool = super::super::db::pool(app).await?;
    let spaces = sqlx::query("SELECT id FROM spaces WHERE kind='space'")
        .fetch_all(pool)
        .await
        .map_err(super::super::db::sql_error)?
        .into_iter()
        .map(|row| {
            let id = row
                .try_get::<String, _>("id")
                .map_err(super::super::db::sql_error)?;
            uuid::Uuid::parse_str(&id)
                .map(|value| value.to_string())
                .map_err(|_| "Space has an invalid identifier".to_owned())
        })
        .collect::<Result<HashSet<_>, _>>()?;
    let mut entries = tokio::fs::read_dir(&root)
        .await
        .map_err(|error| error.to_string())?;
    while let Some(entry) = entries
        .next_entry()
        .await
        .map_err(|error| error.to_string())?
    {
        let id = entry
            .file_name()
            .into_string()
            .map_err(|_| "Agent workspace has an invalid Space identifier".to_owned())?;
        uuid::Uuid::parse_str(&id)
            .map_err(|_| "Agent workspace has an invalid Space identifier".to_owned())?;
        if !spaces.contains(&id) {
            capabilities::agent_artifacts::discard(&entry.path())
                .map_err(|error| error.to_string())?;
        }
    }
    Ok(())
}
