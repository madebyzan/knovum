use super::super::{
    content::{self, PackageFiles},
    contracts::{PluginInstallation, SkillRevision},
    database,
};
use sqlx::SqlitePool;
use tauri::AppHandle;

pub(crate) async fn list(pool: &SqlitePool) -> Result<Vec<SkillRevision>, String> {
    sqlx::query_scalar::<_, String>("SELECT document FROM skills ORDER BY id")
        .fetch_all(pool)
        .await
        .map_err(|error| error.to_string())?
        .iter()
        .map(|document| database::decode(document))
        .collect()
}

pub(crate) async fn load(pool: &SqlitePool, id: &str) -> Result<SkillRevision, String> {
    let document: String = sqlx::query_scalar("SELECT document FROM skills WHERE id=$1")
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(|error| error.to_string())?
        .ok_or("skill-not-found")?;
    database::decode(&document)
}

pub(super) async fn save(pool: &SqlitePool, skill: &SkillRevision) -> Result<(), String> {
    sqlx::query("INSERT INTO skills(id,document) VALUES($1,$2) ON CONFLICT(id) DO UPDATE SET document=excluded.document")
        .bind(&skill.id).bind(database::encode(skill)?).execute(pool).await.map_err(|error| error.to_string())?;
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
    .await?;
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
