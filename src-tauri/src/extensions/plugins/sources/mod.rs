mod archive;
mod git;
mod git_http;
mod network;
use super::super::{
    content::{self, PackageFiles},
    contracts::PluginSource,
};
pub(super) use archive::read_bounded;
use std::path::Path;
use tokio_util::sync::CancellationToken;

pub(crate) async fn read(
    source: &PluginSource,
    cancellation: &CancellationToken,
) -> Result<(PackageFiles, String), String> {
    match source {
        PluginSource::Directory { path } => {
            let path = path.clone();
            let cancellation = cancellation.clone();
            let files =
                crate::blocking::run(move || directory_files(Path::new(&path), &cancellation))
                    .await?;
            let revision = content::files_digest(&files);
            Ok((files, revision))
        }
        PluginSource::Zip { path } => {
            let path = path.clone();
            let cancellation = cancellation.clone();
            crate::blocking::run(move || archive::read_file(Path::new(&path), &cancellation)).await
        }
        PluginSource::HttpsZip { url, sha256 } => {
            let bytes = network::download(url, 64 * 1024 * 1024).await?;
            let digest = content::digest(&bytes);
            if sha256.as_ref().is_some_and(|expected| expected != &digest) {
                return Err("package-download-digest-mismatch".into());
            }
            let cancellation = cancellation.clone();
            let files =
                crate::blocking::run(move || archive::extract(bytes, &cancellation)).await?;
            Ok((files, digest))
        }
        PluginSource::Git {
            url,
            reference,
            subdirectory,
        } => {
            let (parsed, addresses) = network::resolve_url(url).await?;
            let (url, reference, subdirectory) =
                (parsed.to_string(), reference.clone(), subdirectory.clone());
            let cancellation = cancellation.clone();
            crate::blocking::run(move || {
                git::read(&url, &reference, &subdirectory, addresses, cancellation)
            })
            .await
        }
    }
}

pub(crate) fn directory_files(
    root: &Path,
    cancellation: &CancellationToken,
) -> Result<PackageFiles, String> {
    if !std::fs::symlink_metadata(root)
        .map_err(|error| error.to_string())?
        .is_dir()
    {
        return Err("package-source-must-be-directory".into());
    }
    let mut files = PackageFiles::new();
    let mut total = 0usize;
    for entry in walkdir::WalkDir::new(root).follow_links(false) {
        if cancellation.is_cancelled() {
            return Err(super::operations::CANCELLED.into());
        }
        let entry = entry.map_err(|error| error.to_string())?;
        if entry.depth() == 0 {
            continue;
        }
        if entry.file_type().is_dir() {
            continue;
        }
        if !entry.file_type().is_file() {
            return Err("package-special-file-not-supported".into());
        }
        let metadata = entry.metadata().map_err(|error| error.to_string())?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            if metadata.nlink() != 1 {
                return Err("package-hardlink-not-supported".into());
            }
        }
        if metadata.len() > content::MAX_FILE_BYTES as u64 {
            return Err("package-file-size-limit".into());
        }
        let path = entry
            .path()
            .strip_prefix(root)
            .map_err(|error| error.to_string())?
            .to_str()
            .ok_or("package-path-must-be-utf8")?
            .replace(std::path::MAIN_SEPARATOR, "/");
        content::validate_path(&path)?;
        let bytes = archive::read_bounded(entry.path(), content::MAX_FILE_BYTES)?;
        total += bytes.len();
        if total > content::MAX_PACKAGE_BYTES || files.len() >= content::MAX_FILES {
            return Err("package-size-limit-exceeded".into());
        }
        files.insert(path, bytes);
    }
    content::validate_files(&files)?;
    Ok(files)
}

pub(crate) async fn download(url: &str, limit: usize) -> Result<Vec<u8>, String> {
    network::download(url, limit).await
}

#[cfg(test)]
#[path = "tests/archive.rs"]
mod archive_tests;

#[cfg(test)]
#[path = "tests/network.rs"]
mod network_tests;
