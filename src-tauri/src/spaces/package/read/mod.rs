mod extract;

use std::{
    collections::{HashMap, HashSet},
    fs,
    io::Read,
    path::Path,
};

use tauri::ipc::Channel;
use tokio_util::sync::CancellationToken;
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
    target_space: &str,
    cancellation: &CancellationToken,
) -> Result<PackageImport, crate::foundation::app_error::AppError> {
    super::control::check(cancellation)?;
    let file = fs::File::open(source).map_err(|error| error.to_string())?;
    let mut archive = ZipArchive::new(file).map_err(|error| error.to_string())?;
    let total = validation::validate_archive(&mut archive, cancellation)?;
    let manifest = read_text_entry(&mut archive, "manifest.json", MAX_MANIFEST, cancellation)?;
    validation::validate_manifest_version(&manifest)?;
    let index: Vec<AssetIndex> = serde_json::from_str(&read_text_entry(
        &mut archive,
        "assets-index.json",
        MAX_MANIFEST,
        cancellation,
    )?)
    .map_err(|error| error.to_string())?;
    let manifest_assets = super::manifest_assets::parse_cancellable(&manifest, cancellation)?;
    validate_index(&index, &manifest_assets, cancellation)?;
    let assets = extract::extract_assets(
        app,
        &mut archive,
        &index,
        progress,
        total,
        target_space,
        cancellation,
    )?;
    Ok(PackageImport {
        manifest,
        assets: assets.assets,
        namespace: assets.namespace,
    })
}

fn validate_index(
    index: &[AssetIndex],
    manifest: &ManifestAssets,
    cancellation: &CancellationToken,
) -> Result<(), crate::foundation::app_error::AppError> {
    if index.len() > MAX_ASSETS {
        return Err("Space package contains too many assets".into());
    }
    let mut tokens = HashSet::new();
    let mut revision_by_token = HashMap::new();
    for asset in index {
        super::control::check(cancellation)?;
        validation::validate_token(&asset.token)?;
        validation::validate_basename(&asset.name)?;
        if !tokens.insert(asset.token.clone()) {
            return Err("Space package contains duplicate asset tokens".into());
        }
        match asset.source_revision.as_deref() {
            Some(value) if value.starts_with("sha256:") && value.len() == 71 => {
                revision_by_token.insert(asset.token.as_str(), value);
            }
            _ => return Err("Space package asset index requires a SHA-256 sourceRevision".into()),
        }
    }
    if tokens != manifest.used_tokens {
        return Err(
            "Space package asset index must exactly match assets referenced by the manifest".into(),
        );
    }
    for (token, expected) in &manifest.source_revision_by_token {
        if revision_by_token.get(token.as_str()).copied() != Some(expected.as_str()) {
            return Err("Packaged asset revision does not match its indexed asset digest".into());
        }
    }
    Ok(())
}

fn read_text_entry(
    archive: &mut ZipArchive<fs::File>,
    name: &str,
    limit: u64,
    cancellation: &CancellationToken,
) -> Result<String, crate::foundation::app_error::AppError> {
    let mut entry = archive
        .by_name(name)
        .map_err(|_| format!("Space package is missing {name}"))?;
    if entry.size() > limit {
        return Err(format!("{name} exceeds the package limit").into());
    }
    let mut bytes = Vec::with_capacity(entry.size() as usize);
    let mut buffer = [0_u8; 256 * 1024];
    loop {
        super::control::check(cancellation)?;
        let count = entry.read(&mut buffer).map_err(|error| error.to_string())?;
        if count == 0 {
            break;
        }
        bytes.extend_from_slice(&buffer[..count]);
    }
    String::from_utf8(bytes).map_err(|error| error.to_string().into())
}
