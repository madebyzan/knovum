use std::{
    collections::{BTreeMap, BTreeSet},
    fs::File,
    path::{Path, PathBuf},
};
use zip::ZipArchive;

use super::{
    archive_format,
    contracts::*,
    database, files,
    paths::{self, AppPaths},
    settings, staging,
};

pub async fn create(paths: &AppPaths, destination: &Path) -> Result<Option<String>, String> {
    let stage = staging::create_unique(&paths.maintenance, "backup-stage")?;
    let outcome = create_inner(paths, destination, &stage).await;
    let cleanup = staging::cleanup_backup(&stage);
    match outcome {
        Err(error) => {
            if let Err(cleanup_error) = cleanup {
                return Err(format!(
                    "{error}; backup staging cleanup failed: {cleanup_error}"
                ));
            }
            Err(error)
        }
        Ok(()) => Ok(cleanup.err().map(|error| {
            format!("Backup was created, but temporary staging data could not be removed: {error}")
        })),
    }
}

async fn create_inner(paths: &AppPaths, destination: &Path, staging: &Path) -> Result<(), String> {
    let mut entries = Vec::new();
    let mut roots = BTreeMap::new();
    for (name, archive_path) in database::DATABASES {
        let source = paths.config.join(name);
        let present = managed_file_exists(&source)?;
        roots.insert((*archive_path).into(), present);
        if present {
            let staged = staging.join(name);
            database::snapshot(name, &source, &staged).await?;
            entries.push(file_entry(archive_path, staged)?);
        }
    }
    let store = paths.data.join("settings.json");
    let store_present = managed_file_exists(&store)?;
    roots.insert("data/store/settings.json".into(), store_present);
    if store_present {
        settings::validate(&store)?;
        entries.push(file_entry("data/store/settings.json", store)?);
    }
    append_assets(
        &mut entries,
        &mut roots,
        &paths.data.join("spaces/assets"),
        "data/assets/spaces",
    )?;
    append_assets(
        &mut entries,
        &mut roots,
        &paths.data.join("agent-extensions"),
        "data/assets/extensions",
    )?;
    entries.sort_by(|left, right| left.0.path.cmp(&right.0.path));
    let total = entries
        .iter()
        .try_fold(0u64, |total, entry| total.checked_add(entry.0.size))
        .ok_or("Backup size overflow")?;
    if entries.len() > MAX_FILES || total > MAX_TOTAL_BYTES {
        return Err("Managed data exceeds backup safety limits".into());
    }
    let (source_config_root, source_data_root) = paths.canonical_roots()?;
    let manifest = Manifest {
        schema_version: ARCHIVE_SCHEMA,
        application_id: APPLICATION_ID.into(),
        source_config_root,
        source_data_root,
        baselines: database::baselines(),
        roots,
        files: entries.iter().map(|entry| entry.0.clone()).collect(),
    };
    archive_format::write(destination, &manifest, &entries)
}

pub fn extract(paths: &AppPaths, source: &Path) -> Result<(Manifest, PathBuf, PathBuf), String> {
    let source_metadata = std::fs::symlink_metadata(source).map_err(|error| error.to_string())?;
    if !source_metadata.is_file() || source_metadata.file_type().is_symlink() {
        return Err("Backup source must remain a regular non-symbolic-link file".into());
    }
    let token = uuid::Uuid::new_v4();
    let config_stage = paths.config.join(format!(".restore-stage-{token}-config"));
    let data_stage = paths.data.join(format!(".restore-stage-{token}-data"));
    std::fs::create_dir(&config_stage).map_err(|error| error.to_string())?;
    if let Err(error) = std::fs::create_dir(&data_stage) {
        if let Err(cleanup_error) = staging::cleanup_restore(&config_stage, &data_stage) {
            return Err(format!(
                "{error}; restore staging cleanup failed: {cleanup_error}"
            ));
        }
        return Err(error.to_string());
    }
    let outcome = extract_inner(paths, source, &config_stage, &data_stage);
    match outcome {
        Ok(manifest) => Ok((manifest, config_stage, data_stage)),
        Err(cause) => match staging::cleanup_restore(&config_stage, &data_stage) {
            Ok(()) => Err(cause),
            Err(cleanup) => Err(format!(
                "{cause}; restore staging cleanup failed: {cleanup}"
            )),
        },
    }
}

fn extract_inner(
    paths: &AppPaths,
    source: &Path,
    config_stage: &Path,
    data_stage: &Path,
) -> Result<Manifest, String> {
    let file = File::open(source).map_err(|error| error.to_string())?;
    let mut archive =
        ZipArchive::new(file).map_err(|error| format!("Backup archive is invalid: {error}"))?;
    if archive.len() > MAX_FILES + 1 {
        return Err("Backup archive has too many entries".into());
    }
    let manifest = archive_format::read_manifest(&mut archive)?;
    archive_format::validate_manifest(paths, &manifest)?;
    let expected: BTreeMap<_, _> = manifest
        .files
        .iter()
        .map(|file| (file.path.as_str(), file))
        .collect();
    let mut found = BTreeSet::new();
    let mut manifests = 0usize;
    let mut total = 0u64;
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index).map_err(|error| error.to_string())?;
        let name = entry.name().to_owned();
        if name == MANIFEST_PATH {
            manifests += 1;
            if entry.is_dir() {
                return Err("Backup manifest entry is invalid".into());
            }
            continue;
        }
        files::validate_archive_path(&name)?;
        if entry.is_dir() || !matches!(entry.unix_mode().unwrap_or(0) & 0o170000, 0 | 0o100000) {
            return Err(format!("Unsupported backup entry: {name}"));
        }
        if !found.insert(name.to_lowercase()) {
            return Err(format!("Duplicate backup entry: {name}"));
        }
        let declared = expected
            .get(name.as_str())
            .ok_or_else(|| format!("Unexpected backup entry: {name}"))?;
        total = total
            .checked_add(declared.size)
            .ok_or("Backup size overflow")?;
        if total > MAX_TOTAL_BYTES || entry.size() != declared.size {
            return Err("Backup size validation failed".into());
        }
        let destination = staged_path(&config_stage, &data_stage, &name)?;
        let digest = files::copy_bounded(&mut entry, &destination, declared.size)?;
        if digest != declared.sha256 {
            return Err(format!("Backup hash validation failed: {name}"));
        }
    }
    if manifests != 1 || found.len() != expected.len() {
        return Err("Backup archive entries do not match the manifest".into());
    }
    Ok(manifest)
}

fn append_assets(
    entries: &mut Vec<(ArchiveFile, PathBuf)>,
    roots: &mut BTreeMap<String, bool>,
    source: &Path,
    archive_root: &str,
) -> Result<(), String> {
    let present = paths::entry_exists(source)?;
    roots.insert(archive_root.into(), present);
    entries.extend(files::collect(source, archive_root)?);
    Ok(())
}

fn managed_file_exists(path: &Path) -> Result<bool, String> {
    if !paths::entry_exists(path)? {
        return Ok(false);
    }
    let metadata = std::fs::symlink_metadata(path).map_err(|error| error.to_string())?;
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err(format!(
            "Managed backup source must be a regular file: {}",
            path.display()
        ));
    }
    Ok(true)
}

fn file_entry(path: &str, source: PathBuf) -> Result<(ArchiveFile, PathBuf), String> {
    let size = source.metadata().map_err(|error| error.to_string())?.len();
    if size > MAX_FILE_BYTES {
        return Err(format!("Managed file exceeds backup safety limit: {path}"));
    }
    Ok((
        ArchiveFile {
            path: path.into(),
            size,
            sha256: files::hash(&source)?,
        },
        source,
    ))
}

fn staged_path(config: &Path, data: &Path, archive_path: &str) -> Result<PathBuf, String> {
    if let Some(value) = archive_path.strip_prefix("data/config/") {
        return Ok(config.join(value));
    }
    if archive_path == "data/store/settings.json" {
        return Ok(data.join("settings.json"));
    }
    if let Some(value) = archive_path.strip_prefix("data/assets/spaces/") {
        return Ok(data.join("spaces/assets").join(value));
    }
    if let Some(value) = archive_path.strip_prefix("data/assets/extensions/") {
        return Ok(data.join("agent-extensions").join(value));
    }
    Err(format!(
        "Backup entry is outside managed roots: {archive_path}"
    ))
}

pub(super) fn ensure_managed_path(path: &str) -> Result<(), String> {
    staged_path(Path::new("."), Path::new("."), path).map(|_| ())
}
