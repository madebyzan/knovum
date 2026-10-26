use std::collections::HashSet;

use sqlx::Row;
use tauri::AppHandle;

pub async fn discard_orphaned_workspaces(app: &AppHandle) -> Result<(), String> {
    for (session_id, space_id) in super::session::fail_interrupted(app).await? {
        let workspace = crate::spaces::agent_artifacts::workspace::session_directory(
            app,
            &space_id,
            &session_id,
        )
        .map_err(|error| error.to_string())?;
        crate::external_agents::artifacts::discard(&workspace)
            .map_err(|error| error.to_string())?;
    }
    let root = super::workspace::root(app).map_err(|error| error.to_string())?;
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
            crate::external_agents::artifacts::discard(&entry.path())
                .map_err(|error| error.to_string())?;
        }
    }
    Ok(())
}
