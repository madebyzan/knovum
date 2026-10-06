use super::super::{
    contracts::{SkillContent, SkillRevision},
    database, ExtensionState,
};
use super::revisions;
use tauri::{AppHandle, Manager};

#[tauri::command]
pub async fn list_skills(app: AppHandle) -> Result<Vec<SkillRevision>, String> {
    super::super::plugins::ensure_builtin(&app).await?;
    super::list(&database::pool(&app).await?).await
}

#[tauri::command]
pub async fn read_skill(app: AppHandle, id: String) -> Result<SkillContent, String> {
    let skill = super::load(&database::pool(&app).await?, &id).await?;
    let files = super::files(&app, &skill).await?;
    let markdown =
        String::from_utf8(files["SKILL.md"].clone()).map_err(|error| error.to_string())?;
    Ok(SkillContent {
        skill,
        markdown,
        files: files.keys().cloned().collect(),
    })
}

#[tauri::command]
pub async fn create_skill(
    app: AppHandle,
    name: String,
    markdown: String,
) -> Result<SkillRevision, String> {
    let state = app.state::<ExtensionState>();
    let _guard = state.content_lock.lock().await;
    let files = [("SKILL.md".into(), markdown.into_bytes())]
        .into_iter()
        .collect();
    let skill = super::review(&files, &name)?;
    revisions::publish(&app, &skill, files).await?;
    super::super::reclamation::collect(&app).await?;
    Ok(skill)
}

#[tauri::command]
pub async fn save_skill_revision(
    app: AppHandle,
    id: String,
    markdown: String,
) -> Result<SkillRevision, String> {
    let state = app.state::<ExtensionState>();
    let _guard = state.content_lock.lock().await;
    let previous = super::load(&database::pool(&app).await?, &id).await?;
    if previous.plugin_id.is_some() {
        return Err("plugin-skill-is-read-only-copy-to-edit".into());
    }
    let mut files = super::files(&app, &previous).await?;
    files.insert("SKILL.md".into(), markdown.into_bytes());
    let mut skill = super::review(&files, &previous.name)?;
    skill.id = previous.id;
    skill.enabled = previous.enabled;
    revisions::publish(&app, &skill, files).await?;
    super::super::reclamation::collect(&app).await?;
    Ok(skill)
}

#[tauri::command]
pub async fn import_skill(app: AppHandle, path: String) -> Result<SkillRevision, String> {
    let state = app.state::<ExtensionState>();
    let _guard = state.content_lock.lock().await;
    let directory = std::path::PathBuf::from(path);
    let name = directory
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or("skill-directory-name-required")?
        .to_owned();
    let files = crate::blocking::run(move || {
        super::super::plugins::sources::directory_files(
            &directory,
            &tokio_util::sync::CancellationToken::new(),
        )
    })
    .await?;
    let skill = super::review(&files, &name)?;
    revisions::publish(&app, &skill, files).await?;
    super::super::reclamation::collect(&app).await?;
    Ok(skill)
}

#[tauri::command]
pub async fn copy_skill(app: AppHandle, id: String) -> Result<SkillRevision, String> {
    let state = app.state::<ExtensionState>();
    let _guard = state.content_lock.lock().await;
    let previous = super::load(&database::pool(&app).await?, &id).await?;
    let files = super::files(&app, &previous).await?;
    let skill = super::review(&files, &previous.name)?;
    revisions::publish(&app, &skill, files).await?;
    super::super::reclamation::collect(&app).await?;
    Ok(skill)
}

#[tauri::command]
pub async fn export_skill(app: AppHandle, id: String, path: String) -> Result<(), String> {
    let skill = super::load(&database::pool(&app).await?, &id).await?;
    let files = super::files(&app, &skill).await?;
    let destination = std::path::PathBuf::from(path).join(&skill.name);
    if destination.exists() {
        return Err("skill-export-destination-exists".into());
    }
    crate::blocking::run(move || super::super::content::publish(&destination, &files)).await
}

#[tauri::command]
pub async fn set_skill_enabled(app: AppHandle, id: String, enabled: bool) -> Result<(), String> {
    let state = app.state::<ExtensionState>();
    let _guard = state.content_lock.lock().await;
    let pool = database::pool(&app).await?;
    let mut skill = super::load(&pool, &id).await?;
    skill.enabled = enabled;
    revisions::save(&pool, &skill).await
}

#[tauri::command]
pub async fn delete_skill(app: AppHandle, id: String) -> Result<(), String> {
    let state = app.state::<ExtensionState>();
    let _guard = state.content_lock.lock().await;
    let pool = database::pool(&app).await?;
    let skill = super::load(&pool, &id).await?;
    if skill.plugin_id.is_some() {
        return Err("plugin-skill-owned-by-plugin".into());
    }
    sqlx::query("DELETE FROM skills WHERE id=$1")
        .bind(id)
        .execute(&pool)
        .await
        .map_err(|error| error.to_string())?;
    super::super::reclamation::collect(&app).await
}

#[tauri::command]
pub async fn read_skill_file(
    app: AppHandle,
    id: String,
    path: String,
) -> Result<super::super::contracts::AgentSkillFile, String> {
    use base64::Engine;
    super::super::content::validate_path(&path)?;
    let skill = super::load(&database::pool(&app).await?, &id).await?;
    let mut files = super::files(&app, &skill).await?;
    let bytes = files.remove(&path).ok_or("skill-resource-not-found")?;
    if bytes.len() > 8 * 1024 * 1024 {
        return Err("skill-resource-exceeds-view-budget-export-package-to-inspect".into());
    }
    let (encoding, content) = match std::str::from_utf8(&bytes) {
        Ok(text) => (
            super::super::contracts::SkillFileEncoding::Utf8,
            text.to_owned(),
        ),
        Err(_) => (
            super::super::contracts::SkillFileEncoding::Base64,
            base64::engine::general_purpose::STANDARD.encode(&bytes),
        ),
    };
    Ok(super::super::contracts::AgentSkillFile {
        path,
        encoding,
        content,
        digest: super::super::content::digest(&bytes),
    })
}
