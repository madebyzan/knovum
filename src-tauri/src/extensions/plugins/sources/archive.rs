use super::super::super::content::{self, PackageFiles};
use std::{
    io::{Cursor, Read},
    path::Path,
};
use tokio_util::sync::CancellationToken;

use crate::app_error::{AppError, AppResult};

pub(in crate::extensions::plugins) fn read_bounded(
    path: &Path,
    limit: usize,
) -> Result<Vec<u8>, String> {
    let file = std::fs::File::open(path).map_err(|error| error.to_string())?;
    let mut bytes = Vec::new();
    file.take(limit as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| error.to_string())?;
    if bytes.len() > limit {
        return Err("package-size-limit-exceeded".into());
    }
    Ok(bytes)
}

pub(super) fn read_file(
    path: &Path,
    cancellation: &CancellationToken,
) -> AppResult<(PackageFiles, String)> {
    let bytes = read_bounded(path, 64 * 1024 * 1024)?;
    let digest = content::digest(&bytes);
    Ok((extract(bytes, cancellation)?, digest))
}

pub(super) fn extract(bytes: Vec<u8>, cancellation: &CancellationToken) -> AppResult<PackageFiles> {
    let mut archive =
        zip::ZipArchive::new(Cursor::new(bytes)).map_err(|error| error.to_string())?;
    if archive.len() > content::MAX_FILES {
        return Err("package-file-count-limit".into());
    }
    let mut files = PackageFiles::new();
    let mut names = std::collections::BTreeSet::new();
    let mut total = 0usize;
    let mut paths = Vec::new();
    for index in 0..archive.len() {
        if cancellation.is_cancelled() {
            return Err(AppError::cancelled("Plugin source reading was cancelled"));
        }
        let mut entry = archive.by_index(index).map_err(|error| error.to_string())?;
        let path = entry.name().trim_end_matches('/').to_owned();
        content::validate_path(&path)?;
        if !names.insert(content::filesystem_identity(&path)) {
            return Err(format!("package-case-collision: {path}").into());
        }
        let file_type = entry.unix_mode().unwrap_or(0) & 0o170000;
        if !matches!(file_type, 0 | 0o040000 | 0o100000) {
            return Err("package-special-file-not-supported".into());
        }
        paths.push((path.clone(), entry.is_dir()));
        if entry.is_dir() {
            continue;
        }
        if entry.size() > content::MAX_FILE_BYTES as u64 {
            return Err("package-file-size-limit".into());
        }
        let mut bytes = Vec::new();
        (&mut entry)
            .take(content::MAX_FILE_BYTES as u64 + 1)
            .read_to_end(&mut bytes)
            .map_err(|error| error.to_string())?;
        total += bytes.len();
        if bytes.len() > content::MAX_FILE_BYTES || total > content::MAX_PACKAGE_BYTES {
            return Err("package-size-limit-exceeded".into());
        }
        files.insert(path, bytes);
    }
    content::validate_paths(
        paths
            .iter()
            .map(|(path, directory)| (path.as_str(), *directory)),
    )?;
    content::validate_files(&files)?;
    Ok(files)
}
