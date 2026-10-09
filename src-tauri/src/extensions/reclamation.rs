use super::{content, database};
use std::{collections::BTreeSet, path::Path};
use tauri::{AppHandle, Manager};

pub(super) async fn collect(app: &AppHandle) -> Result<(), String> {
    let pool = database::pool(app).await?;
    let root = content::root(app)?;
    let mut retained_revision_paths = BTreeSet::new();
    let mut retained_plugin_ids = BTreeSet::new();
    for plugin in super::plugins::list(&pool).await? {
        retained_plugin_ids.insert(plugin.id.clone());
        retained_revision_paths.insert(format!(
            "plugins/{}/revisions/{}",
            plugin.id, plugin.revision.digest
        ));
    }
    for skill in super::skills::list(&pool)
        .await?
        .into_iter()
        .filter(|skill| skill.plugin_id.is_none())
    {
        retained_revision_paths.insert(format!("skills/{}/revisions/{}", skill.id, skill.digest));
    }
    let leases: Vec<String> = sqlx::query_scalar("SELECT document FROM content_leases")
        .fetch_all(&pool)
        .await
        .map_err(|error| error.to_string())?;
    for lease in leases {
        retained_revision_paths.extend(database::decode::<Vec<String>>(&lease)?);
    }
    crate::blocking::run(move || {
        collect_content(&root, &retained_revision_paths, retained_plugin_ids)
    })
    .await
}

fn collect_content(
    root: &Path,
    retained_revision_paths: &BTreeSet<String>,
    mut retained_plugin_ids: BTreeSet<String>,
) -> Result<(), String> {
    for kind in ["plugins", "skills"] {
        let directory = root.join(kind);
        if !directory.exists() {
            continue;
        }
        for entity in std::fs::read_dir(&directory).map_err(|error| error.to_string())? {
            let entity = entity.map_err(|error| error.to_string())?;
            if !entity
                .file_type()
                .map_err(|error| error.to_string())?
                .is_dir()
            {
                continue;
            }
            let revisions = entity.path().join("revisions");
            if !revisions.exists() {
                continue;
            }
            for revision in std::fs::read_dir(&revisions).map_err(|error| error.to_string())? {
                let revision = revision.map_err(|error| error.to_string())?;
                if !revision
                    .file_type()
                    .map_err(|error| error.to_string())?
                    .is_dir()
                {
                    continue;
                }
                let relative = format!(
                    "{kind}/{}/revisions/{}",
                    entity.file_name().to_string_lossy(),
                    revision.file_name().to_string_lossy()
                );
                if !retained_revision_paths.contains(&relative) {
                    std::fs::remove_dir_all(revision.path()).map_err(|error| error.to_string())?;
                } else if kind == "plugins" {
                    retained_plugin_ids.insert(entity.file_name().to_string_lossy().into_owned());
                }
            }
            if std::fs::read_dir(&revisions)
                .map_err(|error| error.to_string())?
                .next()
                .is_none()
            {
                std::fs::remove_dir_all(entity.path()).map_err(|error| error.to_string())?;
            }
        }
    }
    let plugin_data_directory = root.join("plugin-data");
    if plugin_data_directory.exists() {
        for directory in
            std::fs::read_dir(&plugin_data_directory).map_err(|error| error.to_string())?
        {
            let directory = directory.map_err(|error| error.to_string())?;
            if !directory
                .file_type()
                .map_err(|error| error.to_string())?
                .is_dir()
            {
                continue;
            }
            if !retained_plugin_ids.contains(directory.file_name().to_string_lossy().as_ref()) {
                std::fs::remove_dir_all(directory.path()).map_err(|error| error.to_string())?;
            }
        }
    }
    Ok(())
}

pub(crate) async fn release(app: &AppHandle, snapshot_id: &str) -> Result<(), String> {
    let state = app.state::<super::ExtensionState>();
    let _guard = state.content_lock.lock().await;
    sqlx::query("DELETE FROM content_leases WHERE snapshot_id=$1")
        .bind(snapshot_id)
        .execute(&database::pool(app).await?)
        .await
        .map_err(|error| error.to_string())?;
    super::mcp::sessions::release(app, snapshot_id).await;
    collect(app).await
}

#[cfg(test)]
#[path = "tests/reclamation.rs"]
mod tests;
