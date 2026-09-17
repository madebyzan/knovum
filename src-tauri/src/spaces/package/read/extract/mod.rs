mod verify;

use std::{
    collections::{HashMap, HashSet},
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
};

use tauri::ipc::Channel;
use uuid::Uuid;
use zip::ZipArchive;

use super::super::{
    send_progress, validation, AssetIndex, ImportedPackageAsset, SpacePackageProgress,
};

pub(super) fn extract_assets(
    app: &tauri::AppHandle,
    archive: &mut ZipArchive<fs::File>,
    index: &[AssetIndex],
    progress: &Channel<SpacePackageProgress>,
    total: u64,
) -> Result<Vec<ImportedPackageAsset>, String> {
    let root = crate::spaces::assets::asset_directory(app)?;
    let namespace = root.join(format!("import-{}", Uuid::new_v4()));
    fs::create_dir(&namespace).map_err(|error| error.to_string())?;
    let targets = package_targets(&namespace, index)?;
    let extracted = (|| {
        create_package_directories(&targets)?;
        extract_entries(archive, &targets, progress, total)?;
        verify::verify_assets(&targets, index)?;
        Ok(targets
            .iter()
            .map(|(token, (path, _))| ImportedPackageAsset {
                token: token.clone(),
                path: path.to_string_lossy().into_owned(),
            })
            .collect())
    })();
    if extracted.is_err() {
        if let Err(error) = fs::remove_dir_all(&namespace) {
            if error.kind() != std::io::ErrorKind::NotFound {
                log::warn!(
                    "failed to remove Space package import namespace {}: {error}",
                    namespace.display()
                );
            }
        }
    }
    extracted
}

fn package_targets(
    root: &Path,
    index: &[AssetIndex],
) -> Result<HashMap<String, (PathBuf, bool)>, String> {
    let mut targets = HashMap::new();
    for item in index {
        let target = root.join(format!("{}-{}", item.token, item.name));
        if target.exists() {
            return Err("Package import namespace unexpectedly contains an existing asset".into());
        }
        targets.insert(item.token.clone(), (target, item.directory));
    }
    Ok(targets)
}

fn create_package_directories(targets: &HashMap<String, (PathBuf, bool)>) -> Result<(), String> {
    for (target, directory) in targets.values() {
        if *directory {
            fs::create_dir(target).map_err(|error| error.to_string())?;
        }
    }
    Ok(())
}

fn extract_entries(
    archive: &mut ZipArchive<fs::File>,
    targets: &HashMap<String, (PathBuf, bool)>,
    progress: &Channel<SpacePackageProgress>,
    total: u64,
) -> Result<(), String> {
    let mut processed = 0_u64;
    let mut seen_file_tokens = HashSet::new();
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index).map_err(|error| error.to_string())?;
        let entry_size = entry.size();
        let Some((token, relative)) = asset_entry(entry.name())? else {
            processed += entry_size;
            send_progress(progress, "reading", processed, total)?;
            continue;
        };
        let (target, directory) = targets
            .get(token)
            .ok_or("Package contains an unindexed asset")?;
        let output = output_path(target, *directory, relative)?;
        if !*directory && !entry.is_dir() {
            seen_file_tokens.insert(token.to_owned());
        }
        if entry.is_dir() {
            fs::create_dir_all(&output).map_err(|error| error.to_string())?;
        } else {
            copy_entry(&mut entry, &output, &mut processed, total, progress)?;
        }
        if entry_size == 0 {
            send_progress(progress, "reading", processed, total)?;
        }
    }
    for (token, (_, directory)) in targets {
        if !*directory && !seen_file_tokens.contains(token) {
            return Err("Package file asset is missing content".into());
        }
    }
    send_progress(progress, "reading", total, total)
}

fn output_path(target: &Path, directory: bool, relative: &Path) -> Result<PathBuf, String> {
    validation::validate_relative_path(relative)?;
    if directory {
        return Ok(target.join(relative));
    }
    if relative != Path::new("content") {
        return Err("Package file asset has an invalid entry path".into());
    }
    Ok(target.to_path_buf())
}

fn copy_entry(
    entry: &mut zip::read::ZipFile<'_, fs::File>,
    output: &Path,
    processed: &mut u64,
    total: u64,
    progress: &Channel<SpacePackageProgress>,
) -> Result<(), String> {
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(output)
        .map_err(|error| error.to_string())?;
    let mut buffer = [0_u8; 1024 * 1024];
    loop {
        let count = entry.read(&mut buffer).map_err(|error| error.to_string())?;
        if count == 0 {
            break;
        }
        file.write_all(&buffer[..count])
            .map_err(|error| error.to_string())?;
        *processed += count as u64;
        send_progress(progress, "reading", *processed, total)?;
    }
    Ok(())
}

fn asset_entry(name: &str) -> Result<Option<(&str, &Path)>, String> {
    if !name.starts_with("assets/") {
        return Ok(None);
    }
    let rest = &name[7..];
    let (token, relative) = rest
        .split_once('/')
        .ok_or("Package asset entry is incomplete")?;
    validation::validate_token(token)?;
    let relative = relative.strip_suffix('/').unwrap_or(relative);
    Ok(Some((token, Path::new(relative))))
}
