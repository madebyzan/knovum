use std::path::PathBuf;

use sqlx::{Sqlite, Transaction};
use tauri::AppHandle;

use crate::spaces::db;

pub(super) async fn path(app: &AppHandle, derivation_id: &str) -> Result<PathBuf, String> {
    let pool = db::pool(app).await?;
    let path = sqlx::query_scalar::<_, Option<String>>(
        r#"SELECT json_extract(n.data, '$.assetPath')
           FROM spaces_knowledge_derivations d
           JOIN spaces_knowledge_derivation_inputs i ON i.derivation_id=d.id AND i.position=0 AND i.role='primary'
           JOIN spaces_nodes n ON n.id=i.node_id AND n.space_id=d.space_id
           WHERE d.id=$1 AND d.status='running' AND n.kind='image' AND n.deleted_at IS NULL"#,
    )
    .bind(derivation_id)
    .fetch_optional(pool)
    .await
    .map_err(db::sql_error)?
    .flatten()
    .ok_or_else(|| "Background-removal source is unavailable".to_owned())?;
    crate::spaces::assets::authorize_spaces_asset(app, &PathBuf::from(path))
}

pub(super) async fn revalidate(
    tx: &mut Transaction<'_, Sqlite>,
    derivation_id: &str,
    space_id: &str,
    source_id: &str,
) -> Result<(), String> {
    let matches: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM spaces_knowledge_derivations d JOIN spaces_knowledge_derivation_inputs i ON i.derivation_id=d.id AND i.position=0 AND i.role='primary' JOIN spaces_nodes n ON n.id=i.node_id WHERE d.id=$1 AND n.id=$2 AND n.space_id=$3 AND n.deleted_at IS NULL AND n.kind='image' AND json_extract(n.data,'$.sourceRevision')=i.source_revision",
    )
    .bind(derivation_id)
    .bind(source_id)
    .bind(space_id)
    .fetch_one(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    if matches != 1 {
        return Err("Image source changed during background removal".into());
    }
    Ok(())
}
