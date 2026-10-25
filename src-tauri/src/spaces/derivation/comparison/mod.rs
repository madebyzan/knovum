//! What a derived picture is compared with: the exact picture its derivation read.
//!
//! A derivation reads its source as rendered — the asset with the edit it had at that moment — and
//! records both. The source may be edited again afterwards without changing what an output was made
//! from, so the comparison renders the recorded edit, never today's. Only a replaced asset makes
//! the input unrecoverable: the file that was read is gone, and there is nothing honest to compare.
//!
//! Whether an output compares with its source at all is a property of the transform
//! (`DerivedOutput::compares_with_source`), read back from the action the derivation recorded.

use std::path::{Path, PathBuf};

use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use tauri::{AppHandle, Manager};

use super::{
    image_operations::contracts::ImageAction, input_revision::RevisionBasis, source_snapshot,
};
use crate::foundation::app_error::AppError;
use crate::spaces::db;

/// The picture a derivation read, as a file the renderer can show.
#[derive(Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub struct DerivationComparisonInput {
    path: String,
    /// Names the file's content.
    revision: String,
}

struct Comparable {
    asset_path: String,
    source_revision: String,
    input_revision: String,
    /// The edit the input was rendered with; `null` when the asset was read as it is.
    edit: Value,
}

/// Whether a derived picture can be laid over what it was made from.
#[tauri::command]
pub async fn is_derivation_comparable(app: AppHandle, node_id: String) -> Result<bool, AppError> {
    let pool = db::pool(&app).await?;
    Ok(resolve(pool, &node_id).await?.is_some())
}

/// Materialises the input a comparison shows. An unedited read is the asset itself; an edited
/// one is rendered once into a cache named by its input revision, which already identifies the
/// asset and the edit together.
#[tauri::command]
pub async fn prepare_derivation_comparison(
    app: AppHandle,
    node_id: String,
) -> Result<DerivationComparisonInput, AppError> {
    let pool = db::pool(&app).await?;
    let comparable = resolve(pool, &node_id)
        .await?
        .ok_or("Derived image has no input to compare with")?;
    if comparable.edit.is_null() {
        let path =
            crate::spaces::assets::authorize_spaces_asset(&app, Path::new(&comparable.asset_path))?;
        return Ok(DerivationComparisonInput {
            path: path.to_string_lossy().into_owned(),
            revision: comparable.source_revision,
        });
    }
    let path = rendered_input_path(&app, &comparable.input_revision)?;
    if !path.metadata().is_ok_and(|metadata| metadata.len() > 0) {
        let rendered = source_snapshot::render_pinned_png(
            &app,
            &comparable.asset_path,
            &comparable.source_revision,
            &comparable.edit,
        )
        .await?;
        publish(&path, &rendered.png).await?;
    }
    Ok(DerivationComparisonInput {
        path: path.to_string_lossy().into_owned(),
        revision: comparable.input_revision,
    })
}

async fn resolve(pool: &sqlx::SqlitePool, node_id: &str) -> Result<Option<Comparable>, String> {
    let row: Option<(String, String, String, String, String, String)> = sqlx::query_as(
        "SELECT d.action_kind,i.source_revision,i.revision_basis,i.input_revision,i.input_snapshot,n.data \
         FROM spaces_knowledge_derivations d \
         JOIN spaces_knowledge_derivation_inputs i ON i.derivation_id=d.id AND i.position=0 AND i.role='primary' \
         JOIN spaces_nodes n ON n.id=i.node_id AND n.space_id=d.space_id AND n.kind='image' AND n.deleted_at IS NULL \
         WHERE d.output_node_id=$1 AND d.status='completed' \
         ORDER BY d.completed_at DESC LIMIT 1",
    )
    .bind(node_id)
    .fetch_optional(pool)
    .await
    .map_err(db::sql_error)?;
    let Some((action, source_revision, basis, input_revision, snapshot, data)) = row else {
        return Ok(None);
    };
    if !ImageAction::recorded(&action).is_some_and(|action| action.output().compares_with_source())
    {
        return Ok(None);
    }
    if RevisionBasis::parse(&basis)? != RevisionBasis::Rendered {
        return Err("An image derivation reads its source as rendered".into());
    }
    let data: Value = serde_json::from_str(&data).map_err(db::json_error)?;
    if data.get("sourceRevision").and_then(Value::as_str) != Some(source_revision.as_str()) {
        return Ok(None);
    }
    let asset_path = data
        .get("assetPath")
        .and_then(Value::as_str)
        .ok_or("Image derivation source asset is unavailable")?
        .to_owned();
    let snapshot: Value = serde_json::from_str(&snapshot).map_err(db::json_error)?;
    Ok(Some(Comparable {
        asset_path,
        source_revision,
        input_revision,
        edit: snapshot.get("imageEdit").cloned().unwrap_or(Value::Null),
    }))
}

fn rendered_input_path(app: &AppHandle, input_revision: &str) -> Result<PathBuf, String> {
    let directory = app
        .path()
        .app_cache_dir()
        .map_err(|error| error.to_string())?
        .join("derivation-inputs");
    std::fs::create_dir_all(&directory).map_err(|error| error.to_string())?;
    let name = hex::encode(Sha256::digest(input_revision));
    Ok(directory.join(format!("{name}.png")))
}

/// Content-addressed, so a concurrent render of the same input writes the same bytes; the rename
/// keeps a reader from ever seeing a partial file.
async fn publish(path: &Path, png: &[u8]) -> Result<(), String> {
    let partial = path.with_extension(format!("{}.part", uuid::Uuid::new_v4()));
    let written = match tokio::fs::write(&partial, png).await {
        Ok(()) => tokio::fs::rename(&partial, path).await,
        Err(error) => Err(error),
    };
    if let Err(error) = written {
        let _ = tokio::fs::remove_file(&partial).await;
        return Err(error.to_string());
    }
    Ok(())
}

#[cfg(test)]
mod tests;
