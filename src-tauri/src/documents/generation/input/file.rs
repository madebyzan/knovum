use std::{fs::File, io::Read};

use sha2::{Digest, Sha256};

use super::{DocumentFileSource, DocumentFormat};

pub(super) fn read(source: &DocumentFileSource) -> Result<Vec<u8>, String> {
    source.validate()?;
    let metadata = std::fs::symlink_metadata(&source.path).map_err(|error| error.to_string())?;
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err("Document source must be a regular file".into());
    }
    if metadata.len() > 128 * 1024 * 1024 {
        return Err("Document source exceeds 128 MiB".into());
    }
    let mut bytes = Vec::new();
    File::open(&source.path)
        .map_err(|error| error.to_string())?
        .take(128 * 1024 * 1024 + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| error.to_string())?;
    if bytes.len() > 128 * 1024 * 1024 {
        return Err("Document source exceeds 128 MiB".into());
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
    if source
        .path
        .extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case("pdf"))
    {
        let markdown = super::pdf::extract_markdown(&bytes)?;
        let content = super::super::document::DocumentSource {
            title: if matches!(format, DocumentFormat::Pptx) {
                source.title.clone()
            } else {
                String::new()
            },
            markdown,
            citations: vec![],
        };
        return match format {
            DocumentFormat::Html => super::super::html::render(&content, &source.title),
            _ => format.render(&content),
        };
    }
    super::office::render(source, bytes, format)
}
