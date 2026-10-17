use std::{collections::BTreeSet, path::PathBuf};

use crate::foundation::app_error::AppError;
use crate::spaces::assets::{
    access, enrichment, external_open, files,
    image::{document, preview},
    reclamation,
};

#[tauri::command]
pub async fn refresh_asset(
    app: tauri::AppHandle,
    space_id: String,
    node_id: String,
) -> Result<Option<serde_json::Value>, AppError> {
    enrichment::refresh::refresh_node(&app, space_id, node_id).await
}

#[tauri::command]
pub async fn open_asset(app: tauri::AppHandle, source_path: String) -> Result<(), AppError> {
    Ok((crate::foundation::blocking::run(move || {
        files::open_asset(&app, PathBuf::from(source_path))
    })
    .await)?)
}

#[tauri::command]
pub async fn preview_asset(app: tauri::AppHandle, source_path: String) -> Result<(), AppError> {
    let source = access::authorize_spaces_asset(&app, &PathBuf::from(source_path))?;
    Ok((crate::desktop::system_preview::open(app, source).await)?)
}

#[tauri::command]
pub async fn preview_image(
    app: tauri::AppHandle,
    space_id: String,
    node_id: String,
) -> Result<(), AppError> {
    let document = document::load(&app, &space_id, &node_id).await?;
    let render_app = app.clone();
    let path =
        crate::foundation::blocking::run(move || preview::prepare(&render_app, document)).await?;
    Ok((crate::desktop::system_preview::open(app, path).await)?)
}

#[tauri::command]
pub async fn open_asset_at_location(
    app: tauri::AppHandle,
    source_path: String,
    location: external_open::ExternalLocation,
) -> Result<(), AppError> {
    Ok((crate::foundation::blocking::run(move || {
        let canonical = access::authorize_spaces_asset(&app, &PathBuf::from(source_path))?;
        external_open::open_at_location(&canonical, location)
    })
    .await)?)
}

#[tauri::command]
pub async fn duplicate_asset(
    app: tauri::AppHandle,
    source_path: String,
) -> Result<String, AppError> {
    Ok((crate::foundation::blocking::run(move || {
        files::duplicate_asset(&app, PathBuf::from(source_path))
    })
    .await)?)
}

#[tauri::command]
pub async fn remove_assets(
    app: tauri::AppHandle,
    source_paths: Vec<String>,
) -> Result<(), AppError> {
    let candidates: BTreeSet<_> = source_paths.into_iter().collect();
    if candidates.len() > 10_000 {
        return Err("asset-cleanup-invalid: too many cleanup candidates".into());
    }
    let pool = crate::spaces::db::pool(&app).await?;
    let removable = reclamation::unreferenced_paths(pool, candidates.clone()).await?;
    let retained = candidates.len() - removable.len();
    Ok((crate::foundation::blocking::run(move || {
        let mut failures = Vec::new();
        if retained > 0 {
            failures.push(format!(
                "{retained} assets are still referenced by durable state"
            ));
        }
        for path in removable {
            if let Err(error) = files::remove_asset(&app, PathBuf::from(path)) {
                failures.push(error);
            }
        }
        if failures.is_empty() {
            Ok(())
        } else {
            Err(format!("asset-cleanup-failed: {}", failures.join("; ")))
        }
    })
    .await)?)
}
