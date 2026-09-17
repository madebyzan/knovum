use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

use super::super::{
    validation::{self, MAX_ASSETS, MAX_ENTRIES, MAX_MANIFEST, MAX_UNPACKED},
    AssetIndex, PackageAsset,
};

pub(super) struct AssetPlan {
    pub(super) token: String,
    pub(super) source: PathBuf,
    pub(super) index: AssetIndex,
    pub(super) bytes: u64,
}

pub(super) fn plan_assets(
    app: &tauri::AppHandle,
    manifest: &str,
    assets: &[PackageAsset],
    manifest_assets: &super::super::manifest_assets::ManifestAssets,
) -> Result<Vec<AssetPlan>, String> {
    if manifest.len() as u64 > MAX_MANIFEST || assets.len() > MAX_ASSETS {
        return Err("Space package exceeds export limits".into());
    }
    let mut tokens = HashSet::new();
    let mut plans = Vec::with_capacity(assets.len());
    let mut entries = 2_usize;
    let mut unpacked = manifest.len() as u64;
    for asset in assets {
        validation::validate_token(&asset.token)?;
        if !tokens.insert(asset.token.clone()) {
            return Err("Duplicate package asset token".into());
        }
        let source = crate::spaces::assets::authorize_spaces_asset(app, Path::new(&asset.path))?;
        let name = source
            .file_name()
            .and_then(|value| value.to_str())
            .ok_or("Package asset has no usable name")?
            .to_owned();
        validation::validate_basename(&name)?;
        let (asset_entries, bytes) = inspect_asset(&source)?;
        entries = entries
            .checked_add(asset_entries)
            .ok_or("Space package entry count overflow")?;
        unpacked = unpacked
            .checked_add(bytes)
            .ok_or("Space package size overflow")?;
        if entries > MAX_ENTRIES || unpacked > MAX_UNPACKED {
            return Err("Space package exceeds export limits".into());
        }
        let source_revision = crate::file_inspection::content_revision(&source)?;
        validate_source_revision(manifest_assets, &asset.token, &source_revision)?;
        let index = AssetIndex {
            token: asset.token.clone(),
            name,
            directory: source.is_dir(),
            source_revision: Some(source_revision),
        };
        plans.push(AssetPlan {
            token: asset.token.clone(),
            source,
            index,
            bytes,
        });
    }
    if tokens != manifest_assets.used_tokens {
        return Err(
            "Space package export assets must exactly match assets referenced by the manifest"
                .into(),
        );
    }
    Ok(plans)
}

fn validate_source_revision(
    manifest_assets: &super::super::manifest_assets::ManifestAssets,
    token: &str,
    actual: &str,
) -> Result<(), String> {
    if let Some(expected) = manifest_assets.source_revisions.get(token) {
        if expected != actual {
            return Err("A managed Space asset changed after its sourceRevision was recorded; refresh it before exporting the Space".into());
        }
    }
    Ok(())
}

fn inspect_asset(source: &Path) -> Result<(usize, u64), String> {
    if source.is_file() {
        return Ok((
            1,
            fs::metadata(source)
                .map_err(|error| error.to_string())?
                .len(),
        ));
    }
    let mut stack = vec![source.to_path_buf()];
    let (mut entries, mut bytes) = (0_usize, 0_u64);
    while let Some(directory) = stack.pop() {
        let mut empty = true;
        for entry in fs::read_dir(&directory).map_err(|error| error.to_string())? {
            empty = false;
            let entry = entry.map_err(|error| error.to_string())?;
            let meta = fs::symlink_metadata(entry.path()).map_err(|error| error.to_string())?;
            if meta.file_type().is_symlink() {
                return Err("Space packages do not export symbolic links".into());
            }
            if meta.is_dir() {
                stack.push(entry.path());
            } else if meta.is_file() {
                entries += 1;
                bytes = bytes
                    .checked_add(meta.len())
                    .ok_or("Space package size overflow")?;
            } else {
                return Err("Space packages accept regular files and directories only".into());
            }
        }
        if empty && directory != source {
            entries += 1;
        }
    }
    Ok((entries, bytes))
}

pub(super) fn verify_unchanged(plans: &[AssetPlan]) -> Result<(), String> {
    for plan in plans {
        let expected = plan
            .index
            .source_revision
            .as_deref()
            .ok_or("Space package asset digest is missing")?;
        if crate::file_inspection::content_revision(&plan.source)? != expected {
            return Err("A Space asset changed while the Space package was being written; retry after refreshing it".into());
        }
    }
    Ok(())
}
