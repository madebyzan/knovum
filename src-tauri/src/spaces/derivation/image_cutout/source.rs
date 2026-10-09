use std::{fs, io::ErrorKind, path::Path};

use serde_json::Value;
use sqlx::{Sqlite, Transaction};
use tauri::AppHandle;

use crate::spaces::{assets::authorize_spaces_asset, db};

pub(super) async fn available(
    app: &AppHandle,
    space_id: &str,
    source_id: &str,
    source_revision: &str,
    source_edit: &Value,
) -> Result<bool, String> {
    let pool = db::pool(app).await?;
    let encoded: String = sqlx::query_scalar(
        "SELECT data FROM spaces_nodes WHERE id=$1 AND space_id=$2 AND kind='image' AND deleted_at IS NULL",
    )
    .bind(source_id)
    .bind(space_id)
    .fetch_optional(pool)
    .await
    .map_err(db::sql_error)?
    .ok_or("Image source is unavailable")?;
    let data: Value = serde_json::from_str(&encoded).map_err(db::json_error)?;
    validate_current(&data, source_revision, source_edit)?;
    asset_available(app, &data)
}

pub(super) fn validate_current(
    payload: &Value,
    source_revision: &str,
    source_edit: &Value,
) -> Result<(), String> {
    if payload.get("sourceRevision").and_then(Value::as_str) != Some(source_revision)
        || payload.get("imageEdit").unwrap_or(&Value::Null) != source_edit
    {
        return Err("Image source changed before background removal started".into());
    }
    Ok(())
}

pub(super) fn asset_available(app: &AppHandle, payload: &Value) -> Result<bool, String> {
    let path = payload
        .get("assetPath")
        .and_then(Value::as_str)
        .ok_or("Image source path is unavailable")?;
    match fs::metadata(path) {
        Ok(metadata) if metadata.is_file() => {
            authorize_spaces_asset(app, Path::new(path))?;
            Ok(true)
        }
        Ok(_) => Err("Image source asset must be a regular file".into()),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(false),
        Err(error) => Err(format!("Unable to inspect image source asset: {error}")),
    }
}

pub(super) async fn revalidate(
    tx: &mut Transaction<'_, Sqlite>,
    derivation_id: &str,
    space_id: &str,
    source_id: &str,
    source_edit: &Value,
) -> Result<(), String> {
    let data: Option<String> = sqlx::query_scalar(
        "SELECT n.data FROM spaces_knowledge_derivations d JOIN spaces_knowledge_derivation_inputs i ON i.derivation_id=d.id AND i.position=0 AND i.role='primary' JOIN spaces_nodes n ON n.id=i.node_id WHERE d.id=$1 AND n.id=$2 AND n.space_id=$3 AND n.deleted_at IS NULL AND n.kind='image' AND json_extract(n.data,'$.sourceRevision')=i.source_revision",
    )
    .bind(derivation_id)
    .bind(source_id)
    .bind(space_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    let data = data.ok_or("Image source changed during background removal")?;
    let data: Value = serde_json::from_str(&data).map_err(db::json_error)?;
    if data.get("imageEdit").unwrap_or(&Value::Null) != source_edit {
        return Err("Image source changed during background removal".into());
    }
    Ok(())
}
