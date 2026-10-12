use std::{fs::File, io::Read};

use sha2::{Digest, Sha256};

use super::{DocumentFileSource, DocumentFormat};

pub(super) fn read(source: &DocumentFileSource) -> Result<Vec<u8>, String> {
    read_snapshot(source, 128 * 1024 * 1024)
}

pub(crate) fn read_snapshot(source: &DocumentFileSource, limit: u64) -> Result<Vec<u8>, String> {
    if !source.path.is_absolute() || source.source_revision.is_empty() {
        return Err("Document resource path and revision are required".into());
    }
    let metadata = std::fs::symlink_metadata(&source.path).map_err(|error| error.to_string())?;
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err("Document source must be a regular file".into());
    }
    if metadata.len() > limit {
        return Err(format!(
            "Document resource exceeds {} MiB",
            limit / 1024 / 1024
        ));
    }
    let mut bytes = Vec::new();
    File::open(&source.path)
        .map_err(|error| error.to_string())?
        .take(limit + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| error.to_string())?;
    if bytes.len() as u64 > limit {
        return Err(format!(
            "Document resource exceeds {} MiB",
            limit / 1024 / 1024
        ));
    }
    let revision = format!("sha256:{}", hex::encode(Sha256::digest(&bytes)));
    if revision != source.source_revision {
        return Err("Document source content changed before conversion".into());
    }
    Ok(bytes)
}

pub(super) fn render(
    source: &DocumentFileSource,
    format: DocumentFormat,
) -> Result<Vec<u8>, String> {
    let bytes = read(source)?;
    source.validate()?;
    if source
        .path
        .extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case("pdf"))
    {
        return Err("PDF conversion requires a page-qualified PDF input".into());
    }
    super::office::render(source, bytes, format)
}
