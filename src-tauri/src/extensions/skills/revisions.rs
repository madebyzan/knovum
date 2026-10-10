use super::super::{
    content::{self, PackageFiles},
    contracts::{PluginInstallation, SkillRevision},
    database,
};
use sqlx::SqlitePool;
use tauri::AppHandle;

pub(crate) async fn list(pool: &SqlitePool) -> Result<Vec<SkillRevision>, String> {
    sqlx::query_as::<_, (String, Option<String>)>(
        "SELECT skills.document, plugins.document FROM skills
         LEFT JOIN plugins ON plugins.id=json_extract(skills.document,'$.pluginId')
         ORDER BY skills.id",
    )
    .fetch_all(pool)
    .await
    .map_err(|error| error.to_string())?
    .iter()
    .map(|(document, plugin)| decode(document, plugin.as_deref()))
    .collect()
}

pub(crate) async fn load(pool: &SqlitePool, skill_id: &str) -> Result<SkillRevision, String> {
    let (document, plugin): (String, Option<String>) = sqlx::query_as(
        "SELECT skills.document, plugins.document FROM skills
         LEFT JOIN plugins ON plugins.id=json_extract(skills.document,'$.pluginId')
         WHERE skills.id=$1",
    )
    .bind(skill_id)
    .fetch_optional(pool)
    .await
    .map_err(|error| error.to_string())?
    .ok_or("skill-not-found")?;
    decode(&document, plugin.as_deref())
}

fn decode(document: &str, plugin: Option<&str>) -> Result<SkillRevision, String> {
    let mut skill: SkillRevision = database::decode(document)?;
    if skill.plugin_id.is_some() {
        let owner: PluginInstallation = database::decode(plugin.ok_or("skill-owner-not-found")?)?;
        skill.enabled = owner.enabled;
    }
    Ok(skill)
}

pub(super) async fn set_enabled(
    pool: &SqlitePool,
    skill_id: &str,
    enabled: bool,
) -> Result<(), String> {
    let mut skill = load(pool, skill_id).await?;
    if skill.plugin_id.is_some() {
        return Err("plugin-skill-owned-by-plugin".into());
    }
    skill.enabled = enabled;
    save(pool, &skill).await
}

pub(super) async fn save(pool: &SqlitePool, skill: &SkillRevision) -> Result<(), String> {
    sqlx::query("INSERT INTO skills(id,document) VALUES($1,$2) ON CONFLICT(id) DO UPDATE SET document=excluded.document")
        .bind(&skill.id).bind(sqlx::types::Json(skill)).execute(pool).await.map_err(|error| error.to_string())?;
    Ok(())
}

pub(crate) async fn files(app: &AppHandle, skill: &SkillRevision) -> Result<PackageFiles, String> {
    let root = content::root(app)?;
    let directory = if let Some(plugin_id) = &skill.plugin_id {
        let plugin: PluginInstallation =
            super::super::plugins::load(&database::pool(app).await?, plugin_id).await?;
        root.join("plugins")
            .join(plugin_id)
            .join("revisions")
            .join(plugin.revision.digest)
            .join("skills")
            .join(&skill.name)
    } else {
        root.join("skills")
            .join(&skill.id)
            .join("revisions")
            .join(&skill.digest)
    };
    let files = crate::blocking::run(move || {
        super::super::plugins::sources::directory_files(
            &directory,
            &tokio_util::sync::CancellationToken::new(),
        )
    })
    .await
    .map_err(|error| error.message)?;
    if content::files_digest(&files) != skill.digest {
        return Err(format!("skill-content-corrupt: {}", skill.name));
    }
    Ok(files)
}

pub(super) async fn publish(
    app: &AppHandle,
    skill: &SkillRevision,
    files: PackageFiles,
) -> Result<(), String> {
    let directory = content::root(app)?
        .join("skills")
        .join(&skill.id)
        .join("revisions")
        .join(&skill.digest);
    crate::blocking::run(move || content::publish(&directory, &files)).await?;
    save(&database::pool(app).await?, skill).await
}

#[cfg(test)]
#[path = "tests/ownership.rs"]
mod ownership_tests;
