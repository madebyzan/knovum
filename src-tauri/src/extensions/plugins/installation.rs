use super::super::{
    content::{self, PackageFiles},
    contracts::{PluginInstallation, PluginPublication, PluginSource},
    database,
};
use super::packages::ReviewedPackage;
use sqlx::SqlitePool;
use tauri::AppHandle;

pub(crate) async fn list(pool: &SqlitePool) -> Result<Vec<PluginInstallation>, String> {
    sqlx::query_scalar::<_, String>("SELECT document FROM plugins ORDER BY id")
        .fetch_all(pool)
        .await
        .map_err(|error| error.to_string())?
        .iter()
        .map(|document| database::decode(document))
        .collect()
}

pub(crate) async fn load(pool: &SqlitePool, id: &str) -> Result<PluginInstallation, String> {
    let document: String = sqlx::query_scalar("SELECT document FROM plugins WHERE id=$1")
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(|error| error.to_string())?
        .ok_or("plugin-not-found")?;
    database::decode(&document)
}

pub(super) async fn commit(
    app: &AppHandle,
    id: &str,
    source: PluginSource,
    reviewed: ReviewedPackage,
    files: PackageFiles,
    builtin: bool,
    publication: Option<PluginPublication>,
) -> Result<PluginInstallation, String> {
    let pool = database::pool(app).await?;
    let previous = list(&pool)
        .await?
        .into_iter()
        .find(|plugin| plugin.id == id);
    let enabled = previous
        .as_ref()
        .map(|plugin| plugin.enabled)
        .unwrap_or(true);
    let installation = PluginInstallation {
        id: id.into(),
        source,
        revision: reviewed.inspection,
        enabled,
        builtin,
        publication,
    };
    let directory = content::root(app)?
        .join("plugins")
        .join(id)
        .join("revisions")
        .join(&installation.revision.digest);
    crate::blocking::run(move || content::publish(&directory, &files)).await?;
    let components = super::connectors::resolve(app, &installation)?;
    let previous_skills = super::super::skills::list(&pool).await?;
    let mut tx = pool.begin().await.map_err(|error| error.to_string())?;
    sqlx::query("INSERT INTO plugins(id,document) VALUES($1,$2) ON CONFLICT(id) DO UPDATE SET document=excluded.document")
        .bind(id).bind(database::encode(&installation)?).execute(&mut *tx).await.map_err(|error| error.to_string())?;
    sqlx::query("DELETE FROM skills WHERE json_extract(document,'$.pluginId')=$1")
        .bind(id)
        .execute(&mut *tx)
        .await
        .map_err(|error| error.to_string())?;
    for mut skill in reviewed.skills {
        if let Some(previous) = previous_skills.iter().find(|previous| {
            previous.plugin_id.as_deref() == Some(id) && previous.name == skill.name
        }) {
            skill.id = previous.id.clone();
            skill.enabled = previous.enabled;
        }
        skill.plugin_id = Some(id.into());
        sqlx::query("INSERT INTO skills(id,document) VALUES($1,$2)")
            .bind(&skill.id)
            .bind(database::encode(&skill)?)
            .execute(&mut *tx)
            .await
            .map_err(|error| error.to_string())?;
    }
    let revoked = super::connectors::publish_on(
        &mut tx,
        &installation.id,
        &installation.revision.digest,
        &components,
    )
    .await?;
    tx.commit().await.map_err(|error| error.to_string())?;
    super::super::mcp::sessions::revoke_plugin(app, id).await;
    for account in revoked {
        super::super::connectors::commands::disconnect_connector_account(app.clone(), account)
            .await?;
    }
    super::super::connectors::cleanup::collect(app).await?;
    super::super::reclamation::collect(app).await?;
    Ok(installation)
}
