use std::{
    collections::BTreeSet,
    fs::File,
    io::{Read, Write},
    path::{Path, PathBuf},
};
use zip::{write::SimpleFileOptions, CompressionMethod, ZipArchive, ZipWriter};

use super::{
    archive,
    contracts::{
        ArchiveFile, Manifest, APPLICATION_ID, ARCHIVE_SCHEMA, MANIFEST_PATH, MAX_FILES,
        MAX_FILE_BYTES, MAX_MANIFEST_BYTES, MAX_TOTAL_BYTES, ROOTS,
    },
    database, file_publication, files,
    paths::AppPaths,
};

pub fn write(
    destination: &Path,
    manifest: &Manifest,
    entries: &[(ArchiveFile, PathBuf)],
) -> Result<(), String> {
    let manifest_bytes = serde_json::to_vec(manifest).map_err(|error| error.to_string())?;
    if manifest_bytes.len() as u64 > MAX_MANIFEST_BYTES {
        return Err("Backup manifest is too large".into());
    }
    let temporary =
        destination.with_extension(format!("knovum-backup-{}.tmp", uuid::Uuid::new_v4()));
    let file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)
        .map_err(|error| error.to_string())?;
    let _guard = FileGuard(temporary.clone());
    let mut writer = ZipWriter::new(file);
    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .unix_permissions(0o600);
    writer
        .start_file(MANIFEST_PATH, options)
        .map_err(|error| error.to_string())?;
    writer
        .write_all(&manifest_bytes)
        .map_err(|error| error.to_string())?;
    for (entry, source) in entries {
        writer
            .start_file(&entry.path, options)
            .map_err(|error| error.to_string())?;
        let metadata = std::fs::symlink_metadata(source).map_err(|error| error.to_string())?;
        if !metadata.is_file() || metadata.file_type().is_symlink() {
            return Err(format!(
                "Managed backup source is no longer a regular file: {}",
                source.display()
            ));
        }
        let digest = files::copy_hashed(
            &mut File::open(source).map_err(|error| error.to_string())?,
            &mut writer,
            entry.size,
        )?;
        if digest != entry.sha256 {
            return Err(format!("Managed backup source changed: {}", entry.path));
        }
    }
    writer
        .finish()
        .map_err(|error| error.to_string())?
        .sync_all()
        .map_err(|error| error.to_string())?;
    file_publication::replace_file(&temporary, destination)
}

pub fn read_manifest(archive: &mut ZipArchive<File>) -> Result<Manifest, String> {
    let mut entry = archive
        .by_name(MANIFEST_PATH)
        .map_err(|_| "Backup manifest is missing")?;
    let mut bytes = Vec::new();
    entry
        .by_ref()
        .take(MAX_MANIFEST_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| error.to_string())?;
    if bytes.len() as u64 > MAX_MANIFEST_BYTES {
        return Err("Backup manifest is too large".into());
    }
    serde_json::from_slice(&bytes).map_err(|error| format!("Backup manifest is invalid: {error}"))
}

pub fn validate_manifest(paths: &AppPaths, manifest: &Manifest) -> Result<(), String> {
    if manifest.schema_version != ARCHIVE_SCHEMA || manifest.application_id != APPLICATION_ID {
        return Err("Backup format is not supported by this release".into());
    }
    let (config_root, data_root) = paths.canonical_roots()?;
    if manifest.source_config_root != config_root || manifest.source_data_root != data_root {
        return Err("This backup can only be restored to its original Knovum data location; use Space packages to move data between installations".into());
    }
    if manifest.baselines != database::baselines() {
        return Err("Backup database baseline does not match this release".into());
    }
    if manifest.roots.len() != ROOTS.len()
        || ROOTS.iter().any(|root| !manifest.roots.contains_key(*root))
    {
        return Err("Backup root manifest is invalid".into());
    }
    let total = manifest
        .files
        .iter()
        .try_fold(0u64, |total, file| total.checked_add(file.size))
        .ok_or("Backup manifest size overflow")?;
    if manifest.files.len() > MAX_FILES || total > MAX_TOTAL_BYTES {
        return Err("Backup manifest exceeds safety limits".into());
    }
    let mut identities = BTreeSet::new();
    for file in &manifest.files {
        files::validate_archive_path(&file.path)?;
        let digest_valid =
            file.sha256.len() == 64 && file.sha256.bytes().all(|byte| byte.is_ascii_hexdigit());
        if file.path == MANIFEST_PATH
            || !identities.insert(file.path.to_lowercase())
            || file.size > MAX_FILE_BYTES
            || !digest_valid
        {
            return Err("Backup file manifest is invalid".into());
        }
        archive::ensure_managed_path(&file.path)?;
        let root = ROOTS
            .iter()
            .find(|root| file.path == **root || file.path.starts_with(&format!("{root}/")))
            .ok_or("Backup file is outside declared roots")?;
        if manifest.roots.get(*root).copied() != Some(true) {
            return Err(format!(
                "Backup file belongs to an absent root: {}",
                file.path
            ));
        }
    }
    Ok(())
}

struct FileGuard(PathBuf);
impl Drop for FileGuard {
    fn drop(&mut self) {
        if self.0.exists() {
            if let Err(error) = std::fs::remove_file(&self.0) {
                log::warn!("failed to remove incomplete backup archive: {error}");
            }
        }
    }
}
