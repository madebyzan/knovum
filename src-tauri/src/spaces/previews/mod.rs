mod paths;
mod thumbnail;

use std::path::Path;

use tokio_util::sync::CancellationToken;

use super::assets::{enrichment::ensure_active, PreparedAsset};

pub(crate) fn authorize_generated_preview(
    app: &tauri::AppHandle,
    source: &Path,
) -> Result<std::path::PathBuf, String> {
    let previews = paths::preview_directory(app)?
        .canonicalize()
        .map_err(|error| error.to_string())?;
    let canonical = source.canonicalize().map_err(|error| error.to_string())?;
    if canonical == previews || !canonical.starts_with(&previews) {
        return Err("Only Space-generated previews can be accessed".into());
    }
    Ok(canonical)
}

pub(crate) fn enrich_visual(
    app: &tauri::AppHandle,
    source: &Path,
    asset: &mut PreparedAsset,
    cancellation: &CancellationToken,
) -> Result<(), String> {
    if asset.preview_language.as_deref() == Some("html") {
        return Ok(());
    }
    ensure_active(cancellation)?;
    attach_system_thumbnail(app, source, asset, cancellation)
}

pub(crate) fn reserved_thumbnail_path(
    app: &tauri::AppHandle,
    source: &Path,
) -> Result<Option<std::path::PathBuf>, String> {
    thumbnail::reserve(app, source)
}

pub(crate) fn enrich_visual_at(
    app: &tauri::AppHandle,
    source: &Path,
    asset: &mut PreparedAsset,
    cancellation: &CancellationToken,
    reserved: Option<&Path>,
    claim_path: &mut dyn FnMut(&Path) -> Result<(), String>,
) -> Result<(), String> {
    if asset.preview_language.as_deref() == Some("html") {
        return Ok(());
    }
    ensure_active(cancellation)?;
    if asset.preview_path.is_some() {
        return Ok(());
    }
    attach_thumbnail_at(app, source, asset, cancellation, reserved, claim_path)
}

fn attach_system_thumbnail(
    app: &tauri::AppHandle,
    source: &Path,
    asset: &mut PreparedAsset,
    cancellation: &CancellationToken,
) -> Result<(), String> {
    if asset.preview_path.is_some() {
        return Ok(());
    }
    attach_thumbnail(app, source, asset, cancellation)
}

fn attach_thumbnail(
    app: &tauri::AppHandle,
    source: &Path,
    asset: &mut PreparedAsset,
    cancellation: &CancellationToken,
) -> Result<(), String> {
    let Some(path) = thumbnail::generate(app, source, cancellation)? else {
        return Ok(());
    };
    apply_thumbnail(path, asset)
}

fn attach_thumbnail_at(
    app: &tauri::AppHandle,
    source: &Path,
    asset: &mut PreparedAsset,
    cancellation: &CancellationToken,
    reserved: Option<&Path>,
    claim_path: &mut dyn FnMut(&Path) -> Result<(), String>,
) -> Result<(), String> {
    let Some(path) = (match reserved {
        Some(target) => thumbnail::generate_at(source, target, cancellation, claim_path).map(Some),
        None => thumbnail::generate(app, source, cancellation),
    })?
    else {
        return Ok(());
    };
    apply_thumbnail(path, asset)
}

fn apply_thumbnail(path: std::path::PathBuf, asset: &mut PreparedAsset) -> Result<(), String> {
    // The system preview is the visual representation shown on the canvas. Its aspect ratio is
    // authoritative when the source inspector did not provide geometry (notably video and audio).
    if asset.intrinsic_width.is_none() || asset.intrinsic_height.is_none() {
        let (width, height) = image::image_dimensions(&path).map_err(|error| {
            if let Err(cleanup) = std::fs::remove_file(&path) {
                log::warn!("invalid thumbnail cleanup failed: {cleanup}");
            }
            format!("failed to read system thumbnail dimensions: {error}")
        })?;
        asset.intrinsic_width.get_or_insert(width);
        asset.intrinsic_height.get_or_insert(height);
    }
    asset.preview_path = Some(path.to_string_lossy().into_owned());
    Ok(())
}

pub fn remove_generated(app: &tauri::AppHandle, source: &Path) -> Result<(), String> {
    paths::remove_generated_previews(app, source)
}

pub(crate) async fn reclaim_unreferenced(
    app: &tauri::AppHandle,
    candidates: std::collections::BTreeSet<String>,
) -> Result<(), String> {
    let directory = paths::preview_directory(app)?;
    let candidates = candidates
        .into_iter()
        .filter(|path| Path::new(path).starts_with(&directory))
        .collect();
    let pool = crate::spaces::db::pool(app).await?;
    let unreferenced = super::assets::reclamation::unreferenced_paths(pool, candidates).await?;
    for path in unreferenced {
        let canonical = authorize_generated_preview(app, Path::new(&path))?;
        std::fs::remove_file(canonical).map_err(|error| error.to_string())?;
    }
    Ok(())
}
