use super::super::{
    contracts::{ExtensionChange, SkillContent, SkillRevision},
    database, ExtensionState,
};
use super::revisions;
use crate::foundation::app_error::{AppError, AppResult};
use tauri::{AppHandle, Manager};

#[tauri::command]
pub async fn list_skills(app: AppHandle) -> AppResult<Vec<SkillRevision>> {
    super::super::plugins::reconcile_bundled_plugins(&app).await?;
    super::list(&database::pool(&app).await?)
        .await
        .map_err(AppError::from)
}

#[tauri::command]
pub async fn read_skill(app: AppHandle, skill_id: String) -> AppResult<SkillContent> {
    let skill = super::load(&database::pool(&app).await?, &skill_id).await?;
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
) -> AppResult<ExtensionChange<SkillRevision>> {
    let state = app.state::<ExtensionState>();
    let _guard = state.content_lock.lock().await;
    let files = [("SKILL.md".into(), markdown.into_bytes())]
        .into_iter()
        .collect();
    let skill = super::review(&files, &name)?;
    revisions::publish(&app, &skill, files).await?;
    Ok(ExtensionChange::committed(
        skill,
        [super::super::reclamation::collect(&app).await],
    ))
}

#[tauri::command]
pub async fn save_skill_revision(
    app: AppHandle,
    skill_id: String,
    expected_digest: String,
    markdown: String,
) -> AppResult<ExtensionChange<SkillRevision>> {
    let state = app.state::<ExtensionState>();
    let _guard = state.content_lock.lock().await;
    let previous = super::load(&database::pool(&app).await?, &skill_id).await?;
    if previous.digest != expected_digest {
        return Err(AppError::invalid("skill-revised-refresh-before-saving"));
    }
    if previous.plugin_id.is_some() {
        return Err(AppError::invalid("plugin-skill-is-read-only-copy-to-edit"));
    }
    let mut files = super::files(&app, &previous).await?;
    files.insert("SKILL.md".into(), markdown.into_bytes());
    let mut skill = super::review(&files, &previous.name)?;
    skill.id = previous.id;
    skill.enabled = previous.enabled;
    revisions::publish(&app, &skill, files).await?;
    Ok(ExtensionChange::committed(
        skill,
        [super::super::reclamation::collect(&app).await],
    ))
}

#[tauri::command]
pub async fn import_skill(
    app: AppHandle,
    path: String,
) -> AppResult<ExtensionChange<SkillRevision>> {
    let state = app.state::<ExtensionState>();
    let _guard = state.content_lock.lock().await;
    let directory = std::path::PathBuf::from(path);
    let name = directory
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or("skill-directory-name-required")?
        .to_owned();
    // `directory_files` already returns a structured `AppError`; propagate it
    // as-is instead of flattening it down to its bare message string.
    let files = crate::foundation::blocking::run(move || {
        super::super::plugins::sources::directory_files(
            &directory,
            &tokio_util::sync::CancellationToken::new(),
        )
    })
    .await?;
    let skill = super::review(&files, &name)?;
    revisions::publish(&app, &skill, files).await?;
    Ok(ExtensionChange::committed(
        skill,
        [super::super::reclamation::collect(&app).await],
    ))
}

#[tauri::command]
pub async fn duplicate_skill(
    app: AppHandle,
    skill_id: String,
) -> AppResult<ExtensionChange<SkillRevision>> {
    let state = app.state::<ExtensionState>();
    let _guard = state.content_lock.lock().await;
    let previous = super::load(&database::pool(&app).await?, &skill_id).await?;
    let files = super::files(&app, &previous).await?;
    let skill = super::review(&files, &previous.name)?;
    revisions::publish(&app, &skill, files).await?;
    Ok(ExtensionChange::committed(
        skill,
        [super::super::reclamation::collect(&app).await],
    ))
}

#[tauri::command]
pub async fn export_skill(app: AppHandle, skill_id: String, path: String) -> AppResult<()> {
    let skill = super::load(&database::pool(&app).await?, &skill_id).await?;
    let files = super::files(&app, &skill).await?;
    let destination = std::path::PathBuf::from(path).join(&skill.name);
    if destination.exists() {
        return Err(AppError::invalid("skill-export-destination-exists"));
    }
    crate::foundation::blocking::run(move || super::super::content::publish(&destination, &files))
        .await
        .map_err(AppError::from)
}

#[tauri::command]
pub async fn enable_skill(app: AppHandle, skill_id: String) -> AppResult<()> {
    set_enabled(app, skill_id, true)
        .await
        .map_err(AppError::from)
}

#[tauri::command]
pub async fn disable_skill(app: AppHandle, skill_id: String) -> AppResult<()> {
    set_enabled(app, skill_id, false)
        .await
        .map_err(AppError::from)
}

async fn set_enabled(app: AppHandle, skill_id: String, enabled: bool) -> Result<(), String> {
    let state = app.state::<ExtensionState>();
    let _guard = state.content_lock.lock().await;
    let pool = database::pool(&app).await?;
    revisions::set_enabled(&pool, &skill_id, enabled).await
}

#[tauri::command]
pub async fn delete_skill(app: AppHandle, skill_id: String) -> AppResult<ExtensionChange<()>> {
    let state = app.state::<ExtensionState>();
    let _guard = state.content_lock.lock().await;
    let pool = database::pool(&app).await?;
    let skill = super::load(&pool, &skill_id).await?;
    if skill.plugin_id.is_some() {
        return Err(AppError::invalid("plugin-skill-owned-by-plugin"));
    }
    sqlx::query("DELETE FROM skills WHERE id=$1")
        .bind(skill_id)
        .execute(&pool)
        .await
        .map_err(|error| error.to_string())?;
    Ok(ExtensionChange::committed(
        (),
        [super::super::reclamation::collect(&app).await],
    ))
}

#[tauri::command]
pub async fn read_skill_file(
    app: AppHandle,
    skill_id: String,
    path: String,
) -> AppResult<super::super::contracts::AgentSkillFile> {
    use base64::Engine;
    super::super::content::validate_path(&path)?;
    let skill = super::load(&database::pool(&app).await?, &skill_id).await?;
    let mut files = super::files(&app, &skill).await?;
    let bytes = files.remove(&path).ok_or("skill-resource-not-found")?;
    if bytes.len() > 8 * 1024 * 1024 {
        return Err(AppError::invalid(
            "skill-resource-exceeds-view-budget-export-package-to-inspect",
        ));
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
