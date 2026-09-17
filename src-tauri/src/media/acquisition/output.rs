use std::{
    fs,
    io::Read,
    path::{Path, PathBuf},
};

use super::metadata::{is_http_url, YtDlpMetadata};

const MAX_METADATA_BYTES: u64 = 4 * 1024 * 1024;

pub(crate) fn load_metadata(workspace: &Path) -> Result<YtDlpMetadata, String> {
    let file =
        fs::File::open(workspace.join("metadata.json")).map_err(|error| error.to_string())?;
    let mut bytes = Vec::new();
    file.take(MAX_METADATA_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| error.to_string())?;
    if bytes.len() as u64 > MAX_METADATA_BYTES {
        return Err("yt-dlp metadata exceeded the size limit".into());
    }
    serde_json::from_slice(&bytes)
        .map_err(|error| format!("yt-dlp returned invalid media metadata: {error}"))
}

pub(crate) fn downloaded_path(workspace: &Path, value: &str) -> Result<PathBuf, String> {
    let workspace = workspace
        .canonicalize()
        .map_err(|error| error.to_string())?;
    let path = PathBuf::from(value)
        .canonicalize()
        .map_err(|error| format!("yt-dlp did not produce its declared media file: {error}"))?;
    let metadata = fs::symlink_metadata(&path).map_err(|error| error.to_string())?;
    if !metadata.is_file() || metadata.file_type().is_symlink() || path.parent() != Some(&workspace)
    {
        return Err("yt-dlp returned an invalid downloaded media path".into());
    }
    Ok(path)
}

pub(crate) fn extension(path: &Path) -> Result<String, String> {
    let value = path
        .extension()
        .and_then(|value| value.to_str())
        .map(str::to_ascii_lowercase)
        .ok_or("yt-dlp downloaded media without a file extension")?;
    if value.is_empty()
        || value.len() > 16
        || !value.chars().all(|char| char.is_ascii_alphanumeric())
    {
        return Err("yt-dlp returned an invalid media file extension".into());
    }
    Ok(value)
}

pub(crate) fn required_url(value: String, label: &str) -> Result<String, String> {
    if value.len() > 4096 || !is_http_url(&value) {
        return Err(format!("yt-dlp returned an invalid {label}"));
    }
    Ok(value)
}

pub(crate) fn required_text(value: String, label: &str, max: usize) -> Result<String, String> {
    let value = value.trim();
    if value.is_empty() || value.encode_utf16().count() > max {
        return Err(format!("yt-dlp returned an invalid {label}"));
    }
    Ok(value.to_owned())
}

pub(crate) fn optional_text(
    value: Option<String>,
    label: &str,
    max: usize,
) -> Result<Option<String>, String> {
    value
        .map(|value| required_text(value, label, max))
        .transpose()
}
