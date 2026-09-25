use std::{collections::BTreeSet, path::PathBuf};

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
) -> Result<Option<serde_json::Value>, String> {
    enrichment::refresh::refresh_node(&app, space_id, node_id).await
}

#[tauri::command]
pub async fn open_asset(app: tauri::AppHandle, source_path: String) -> Result<(), String> {
    crate::blocking::run(move || files::open_asset(&app, PathBuf::from(source_path))).await
}

#[tauri::command]
pub async fn preview_asset(app: tauri::AppHandle, source_path: String) -> Result<(), String> {
    let source = access::authorize_spaces_asset(&app, &PathBuf::from(source_path))?;
    crate::system_preview::open(app, source).await
}

#[tauri::command]
pub async fn preview_image(
    app: tauri::AppHandle,
    space_id: String,
    node_id: String,
) -> Result<(), String> {
    let document = document::load(&app, &space_id, &node_id).await?;
    let render_app = app.clone();
    let path = crate::blocking::run(move || preview::prepare(&render_app, document)).await?;
    crate::system_preview::open(app, path).await
}

#[tauri::command]
pub async fn open_asset_at_location(
    app: tauri::AppHandle,
    source_path: String,
    location: external_open::ExternalLocation,
) -> Result<(), String> {
    crate::blocking::run(move || {
        let canonical = access::authorize_spaces_asset(&app, &PathBuf::from(source_path))?;
        external_open::open_at_location(&canonical, location)
    })
    .await
}

#[tauri::command]
pub async fn duplicate_asset(app: tauri::AppHandle, source_path: String) -> Result<String, String> {
    crate::blocking::run(move || files::duplicate_asset(&app, PathBuf::from(source_path))).await
}

#[tauri::command]
pub async fn remove_asset(app: tauri::AppHandle, source_path: String) -> Result<(), String> {
    let pool = crate::spaces::db::pool(&app).await?;
    let candidates = BTreeSet::from([source_path.clone()]);
    if reclamation::unreferenced_paths(pool, candidates)
        .await?
        .is_empty()
    {
        return Err("Space asset is still referenced by durable node state".to_owned());
    }
    crate::blocking::run(move || files::remove_asset(&app, PathBuf::from(source_path))).await
}

#[tauri::command]
pub async fn export_asset(
    app: tauri::AppHandle,
    source_path: String,
    destination_path: String,
) -> Result<(), String> {
    crate::blocking::run(move || {
        files::export_asset(
            &app,
            PathBuf::from(source_path),
            PathBuf::from(destination_path),
        )
    })
    .await
}

#[tauri::command]
pub async fn export_text(
    app: tauri::AppHandle,
    destination_path: String,
    text: String,
) -> Result<(), String> {
    crate::blocking::run(move || files::export_text(&app, PathBuf::from(destination_path), text))
        .await
}

#[tauri::command]
pub async fn export_image(
    app: tauri::AppHandle,
    space_id: String,
    node_id: String,
    destination_path: String,
) -> Result<(), String> {
    let document = document::load(&app, &space_id, &node_id).await?;
    crate::blocking::run(move || {
        let rendered =
            crate::images::render_source_png(&app, &document.source_path, &document.edit)?;
        files::export_image(&app, PathBuf::from(destination_path), rendered.png)
    })
    .await
}
