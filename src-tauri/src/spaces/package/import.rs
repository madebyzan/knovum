use super::SpacePackageProgress;
use crate::app_error::AppError;
use serde_json::{json, Value};
use std::{fs, path::PathBuf};
use tauri::{ipc::Channel, AppHandle};

pub(super) async fn execute(
    app: AppHandle,
    source: PathBuf,
    progress: Channel<SpacePackageProgress>,
) -> Result<Value, AppError> {
    let target_id = uuid::Uuid::new_v4().to_string();
    let worker_app = app.clone();
    let worker_id = target_id.clone();
    let unpacked = crate::blocking::run(move || {
        crate::spaces::assets::authorize_source(&worker_app, &source)?;
        super::read::read_package(&worker_app, &source, &progress, &worker_id)
    })
    .await?;
    let restored = async {
        let request = super::remap::prepare(&unpacked.manifest,&unpacked.assets,&target_id)?;
        let overview = json!({"id":request.space.id,"kind":"space","title":request.space.title,"archivedAt":null,"favorite":false,"thumbnailPath":null,"lastOpenedAt":request.space.last_opened_at,"createdAt":request.space.created_at,"updatedAt":request.space.updated_at,"nodeCount":request.nodes.len()});
        super::restore::restore(&app,request).await?;
        Ok::<_,AppError>(overview)
    }.await;
    match restored {
        Ok(overview) => {
            fs::remove_file(unpacked.namespace.join(".pending")).map_err(|e| format!("space-package-import-committed: Space {target_id} was imported but its staging marker could not be finalized: {e}"))?;
            Ok(overview)
        }
        Err(cause) => {
            fs::remove_dir_all(&unpacked.namespace).map_err(|e| {
                format!("Space package restore failed: {cause}; asset cleanup failed: {e}")
            })?;
            Err(cause)
        }
    }
}

pub(crate) async fn reconcile(app: &AppHandle) -> Result<(), String> {
    let root = crate::spaces::assets::asset_directory(app)?;
    let pool = crate::spaces::db::pool(app).await?;
    for entry in fs::read_dir(root).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        if !entry.file_type().map_err(|e| e.to_string())?.is_dir() {
            continue;
        }
        let name = entry.file_name();
        let name = name.to_str().ok_or("Asset namespace name is not UTF-8")?;
        let Some(id) = name.strip_prefix("package-import-") else {
            continue;
        };
        uuid::Uuid::parse_str(id).map_err(|_| "Invalid package import namespace")?;
        let marker = entry.path().join(".pending");
        if !marker.exists() {
            continue;
        }
        let committed: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM spaces WHERE id=$1")
            .bind(id)
            .fetch_one(pool)
            .await
            .map_err(crate::spaces::db::sql_error)?;
        if committed == 1 {
            fs::remove_file(marker).map_err(|e| e.to_string())?;
        } else {
            fs::remove_dir_all(entry.path()).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}
