use std::{
    collections::HashMap,
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
};

use bzip2::read::BzDecoder;
use sha2::{Digest, Sha256};
use tokio_util::sync::CancellationToken;

use super::manifest::{Manifest, ManifestFile};

pub(super) fn extract(
    archive_path: &Path,
    destination: &Path,
    manifest: &Manifest,
    cancellation: &CancellationToken,
) -> Result<(), String> {
    let archive = fs::File::open(archive_path).map_err(|error| error.to_string())?;
    let mut tar = tar::Archive::new(BzDecoder::new(archive));
    let expected = manifest
        .files
        .iter()
        .map(|file| (file.path.as_str(), file))
        .collect::<HashMap<_, _>>();
    let mut extracted = HashMap::with_capacity(expected.len());
    for entry in tar.entries().map_err(|error| error.to_string())? {
        check_cancelled(cancellation)?;
        let mut entry = entry.map_err(|error| error.to_string())?;
        let entry_path = entry
            .path()
            .map_err(|error| error.to_string())?
            .into_owned();
        let relative = strip_root(&entry_path, &manifest.archive_root)?;
        if relative.as_os_str().is_empty() {
            continue;
        }
        let relative_text = relative
            .to_str()
            .ok_or_else(|| "Kokoro archive contains a non-UTF-8 path".to_string())?;
        super::manifest::validate_relative_path(relative_text)?;
        if entry.header().entry_type().is_dir() {
            continue;
        }
        if !entry.header().entry_type().is_file() {
            return Err(format!(
                "Kokoro archive contains a link or special entry: {relative_text}"
            ));
        }
        let file = expected
            .get(relative_text)
            .ok_or_else(|| format!("Kokoro archive contains an unlisted file: {relative_text}"))?;
        if extracted.insert(relative_text.to_owned(), ()).is_some() || entry.size() != file.size {
            return Err(format!(
                "Kokoro archive file metadata differs: {relative_text}"
            ));
        }
        extract_file(&mut entry, &destination.join(relative), file, cancellation)?;
    }
    if extracted.len() != expected.len() {
        let missing = expected
            .keys()
            .find(|path| !extracted.contains_key(**path))
            .copied()
            .unwrap_or("unknown");
        return Err(format!(
            "Kokoro archive is missing a declared file: {missing}"
        ));
    }
    Ok(())
}

fn strip_root(path: &Path, root: &str) -> Result<PathBuf, String> {
    path.strip_prefix(root).map(Path::to_owned).map_err(|_| {
        format!(
            "Kokoro archive entry is outside its declared root: {}",
            path.display()
        )
    })
}

fn extract_file(
    entry: &mut impl Read,
    target: &Path,
    expected: &ManifestFile,
    cancellation: &CancellationToken,
) -> Result<(), String> {
    let parent = target
        .parent()
        .ok_or_else(|| "Kokoro package file has no parent directory".to_string())?;
    fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    let mut output = fs::File::create(target).map_err(|error| error.to_string())?;
    let mut hash = Sha256::new();
    let mut written = 0_u64;
    let mut buffer = [0_u8; 1024 * 1024];
    loop {
        check_cancelled(cancellation)?;
        let read = entry.read(&mut buffer).map_err(|error| error.to_string())?;
        if read == 0 {
            break;
        }
        written = written.saturating_add(read as u64);
        if written > expected.size {
            return Err(format!(
                "Kokoro archive file exceeds its declared size: {}",
                expected.path
            ));
        }
        output
            .write_all(&buffer[..read])
            .map_err(|error| error.to_string())?;
        hash.update(&buffer[..read]);
    }
    output.sync_all().map_err(|error| error.to_string())?;
    let digest = hex::encode(hash.finalize());
    if written != expected.size || digest != expected.sha256 {
        return Err(format!(
            "Kokoro archive file failed verification: {}",
            expected.path
        ));
    }
    Ok(())
}

fn check_cancelled(cancellation: &CancellationToken) -> Result<(), String> {
    if cancellation.is_cancelled() {
        Err("local-resource-preparation-cancelled".to_string())
    } else {
        Ok(())
    }
}
