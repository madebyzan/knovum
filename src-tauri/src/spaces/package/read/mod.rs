mod extract;

use std::{
    collections::{HashMap, HashSet},
    fs,
    io::Read,
    path::Path,
};

use tauri::ipc::Channel;
use zip::ZipArchive;

use super::{
    manifest_assets::ManifestAssets,
    validation::{self, MAX_ASSETS, MAX_MANIFEST},
    AssetIndex, PackageImport, SpacePackageProgress,
};

pub fn read_package(
    app: &tauri::AppHandle,
    source: &Path,
    progress: &Channel<SpacePackageProgress>,
) -> Result<PackageImport, String> {
    let file = fs::File::open(source).map_err(|error| error.to_string())?;
    let mut archive = ZipArchive::new(file).map_err(|error| error.to_string())?;
    let total = validation::validate_archive(&mut archive)?;
    let manifest = read_text_entry(&mut archive, "manifest.json", MAX_MANIFEST)?;
    validation::validate_manifest_version(&manifest)?;
    let index: Vec<AssetIndex> = serde_json::from_str(&read_text_entry(
        &mut archive,
        "assets-index.json",
        MAX_MANIFEST,
    )?)
    .map_err(|error| error.to_string())?;
    let manifest_assets = super::manifest_assets::parse(&manifest)?;
    validate_index(&index, &manifest_assets)?;
    let assets = extract::extract_assets(app, &mut archive, &index, progress, total)?;
    Ok(PackageImport { manifest, assets })
}

fn validate_index(index: &[AssetIndex], manifest: &ManifestAssets) -> Result<(), String> {
    if index.len() > MAX_ASSETS {
        return Err("Space package contains too many assets".into());
    }
    let mut tokens = HashSet::new();
    let mut revisions = HashMap::new();
    for asset in index {
        validation::validate_token(&asset.token)?;
        validation::validate_basename(&asset.name)?;
        if !tokens.insert(asset.token.clone()) {
            return Err("Space package contains duplicate asset tokens".into());
        }
        match asset.source_revision.as_deref() {
            Some(value) if value.starts_with("sha256:") && value.len() == 71 => {
                revisions.insert(asset.token.as_str(), value);
            }
            _ => return Err("Space package asset index requires a SHA-256 sourceRevision".into()),
        }
    }
    if tokens != manifest.used_tokens {
        return Err(
            "Space package asset index must exactly match assets referenced by packaged nodes"
                .into(),
        );
    }
    for (token, expected) in &manifest.source_revisions {
        if revisions.get(token.as_str()).copied() != Some(expected.as_str()) {
            return Err(
                "Packaged node sourceRevision does not match its indexed asset digest".into(),
            );
        }
    }
    Ok(())
}

fn read_text_entry(
    archive: &mut ZipArchive<fs::File>,
    name: &str,
    limit: u64,
) -> Result<String, String> {
    let mut entry = archive
        .by_name(name)
        .map_err(|_| format!("Space package is missing {name}"))?;
    if entry.size() > limit {
        return Err(format!("{name} exceeds the package limit"));
    }
    let mut text = String::new();
    entry
        .read_to_string(&mut text)
        .map_err(|error| error.to_string())?;
    Ok(text)
}
