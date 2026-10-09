use super::SpacePackageProgress;
use crate::app_error::AppError;
use serde_json::{json, Value};
use std::{
    fs,
    path::{Path, PathBuf},
};
use tauri::{ipc::Channel, AppHandle};
use tokio_util::sync::CancellationToken;

pub(super) async fn execute(
    app: AppHandle,
    source: PathBuf,
    progress: Channel<SpacePackageProgress>,
    cancellation: CancellationToken,
    _lease: crate::job_registry::JobLease<CancellationToken>,
) -> Result<Value, AppError> {
    super::control::check(&cancellation)?;
    let target_id = uuid::Uuid::new_v4().to_string();
    let worker_app = app.clone();
    let worker_id = target_id.clone();
    let worker_cancellation = cancellation.clone();
    let unpacked = crate::blocking::run(move || {
        super::control::check(&worker_cancellation)?;
        crate::spaces::assets::authorize_source(&worker_app, &source).map_err(AppError::from)?;
        super::read::read_package(
            &worker_app,
            &source,
            &progress,
            &worker_id,
            &worker_cancellation,
        )
    })
    .await?;
    let restored = async {
        super::control::check(&cancellation)?;
        let request = super::remap::prepare_cancellable(
            &unpacked.manifest,
            &unpacked.assets,
            &target_id,
            &cancellation,
        )?;
        super::control::check(&cancellation)?;
        let overview = json!({"id":request.space.id,"kind":"space","title":request.space.title,"archivedAt":null,"favorite":false,"thumbnailPath":null,"lastOpenedAt":request.space.last_opened_at,"createdAt":request.space.created_at,"updatedAt":request.space.updated_at,"nodeCount":request.nodes.len()});
        super::restore::restore(&app,request,&cancellation).await?;
        Ok::<_,AppError>(overview)
    }.await;
    match restored {
        Ok(overview) => Ok(finalize_committed_import(
            overview,
            &unpacked.namespace.join(".pending"),
            &target_id,
        )),
        Err(mut cause) => {
            if let Err(error) = remove_staging_namespace(&unpacked.namespace) {
                cause = super::control::with_cleanup_error(
                    cause,
                    "Space package import staging cleanup failed",
                    error,
                );
            }
            Err(cause)
        }
    }
}

fn finalize_committed_import<T>(overview: T, marker: &Path, target_id: &str) -> T {
    if let Err(error) = fs::remove_file(marker) {
        log::warn!(
            "Space package {target_id} was imported, but its staging marker could not be removed: {error}"
        );
    }
    overview
}

#[cfg(test)]
#[path = "tests/import.rs"]
mod tests;

pub(crate) async fn reconcile<R: tauri::Runtime>(app: &tauri::AppHandle<R>) -> Result<(), String> {
    let root = crate::spaces::assets::asset_directory(app)?;
    let pool = crate::spaces::db::pool(app).await?;
    for entry in fs::read_dir(root).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let name = entry.file_name();
        let name = name.to_str().ok_or("Asset namespace name is not UTF-8")?;
        let Some(id) = name.strip_prefix("package-import-") else {
            continue;
        };
        let parsed = uuid::Uuid::parse_str(id).map_err(|_| "Invalid package import namespace")?;
        if parsed.to_string() != id {
            return Err("Package import namespace UUID is not canonical".into());
        }
        validate_staging_namespace(&entry.path())?;
        let marker = entry.path().join(".pending");
        let committed: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM spaces WHERE id=$1")
            .bind(id)
            .fetch_one(pool)
            .await
            .map_err(crate::spaces::db::sql_error)?;
        if committed == 1 {
            if marker.exists() {
                fs::remove_file(marker).map_err(|e| e.to_string())?;
            }
        } else {
            remove_staging_namespace(&entry.path())?;
        }
    }
    Ok(())
}

pub(super) fn remove_staging_namespace(path: &Path) -> Result<(), String> {
    validate_staging_namespace(path)?;
    fs::remove_dir_all(path).map_err(|error| error.to_string())
}

fn validate_staging_namespace(path: &Path) -> Result<(), String> {
    let id = path
        .file_name()
        .and_then(|name| name.to_str())
        .and_then(|name| name.strip_prefix("package-import-"))
        .ok_or("Package import namespace is invalid")?;
    let parsed = uuid::Uuid::parse_str(id).map_err(|_| "Package import namespace is invalid")?;
    if parsed.to_string() != id {
        return Err("Package import namespace UUID is not canonical".into());
    }
    let metadata = fs::symlink_metadata(path).map_err(|error| error.to_string())?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err("Package import namespace must be a real directory".into());
    }
    Ok(())
}
