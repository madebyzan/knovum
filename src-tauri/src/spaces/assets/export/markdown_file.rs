use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

use percent_encoding::{percent_decode_str, utf8_percent_encode, AsciiSet, CONTROLS};
use tauri::AppHandle;

use super::files;
use crate::spaces::assets::markdown_media;

const URL_SEGMENT_ENCODE: &AsciiSet = &CONTROLS
    .add(b' ')
    .add(b'"')
    .add(b'#')
    .add(b'%')
    .add(b'(')
    .add(b')')
    .add(b'<')
    .add(b'>')
    .add(b'?')
    .add(b'[')
    .add(b'\\')
    .add(b']')
    .add(b'^')
    .add(b'`')
    .add(b'{')
    .add(b'|')
    .add(b'}');

/// Decodes a Tauri asset protocol URL (`asset://localhost/...`, `http(s)://asset.localhost/...`)
/// or Space asset path and authorizes it strictly within the Space's managed asset directory.
pub(super) fn resolve_managed_asset_url<R: tauri::Runtime>(
    app: &AppHandle<R>,
    raw: &str,
) -> Option<PathBuf> {
    let candidate = decode_asset_path(raw)?;
    let resolved = if candidate.is_absolute() {
        candidate
    } else {
        crate::spaces::assets::asset_directory(app)
            .ok()?
            .join(candidate)
    };
    if fs::symlink_metadata(&resolved)
        .ok()?
        .file_type()
        .is_symlink()
    {
        return None;
    }
    let canonical = crate::spaces::assets::authorize_spaces_asset(app, &resolved).ok()?;
    let metadata = fs::metadata(&canonical).ok()?;
    let limit = if markdown_media::is_video_path(&canonical) {
        markdown_media::MAX_EDITOR_VIDEO_BYTES as u64
    } else {
        crate::documents::generation::images::MAX_IMAGE_BYTES
    };
    if !metadata.is_file() || metadata.len() > limit {
        return None;
    }
    Some(canonical)
}

pub(super) fn is_markdown_destination(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("md") || ext.eq_ignore_ascii_case("markdown"))
}

/// Writes an exported Markdown file. When the document references Space-managed images or videos
/// (such as `asset://localhost/...` editor assets), copies those authorized files into a sibling
/// `<stem>.assets/` directory and rewrites links to portable relative paths so external Markdown
/// viewers display every embedded media file cleanly.
pub(super) fn write_markdown(
    app: &AppHandle,
    destination: &Path,
    text: &str,
) -> Result<(), String> {
    crate::spaces::assets::authorize_destination(app, destination)?;
    let mut managed_by_url: BTreeMap<String, PathBuf> = BTreeMap::new();
    let _ = markdown_media::rewrite_markdown_media(text, |dest_url| {
        if !managed_by_url.contains_key(dest_url) {
            if let Some(canonical) = resolve_managed_asset_url(app, dest_url) {
                managed_by_url.insert(dest_url.to_owned(), canonical);
            }
        }
        Ok(None)
    })?;
    if managed_by_url.is_empty() {
        return files::write(app, destination, text.as_bytes());
    }

    let parent = destination
        .parent()
        .ok_or("Export destination must have a parent directory")?;
    let stem = destination
        .file_stem()
        .and_then(|part| part.to_str())
        .filter(|part| !part.is_empty())
        .unwrap_or("document");
    let assets_dir_name = format!("{stem}.assets");
    let assets_dir = parent.join(&assets_dir_name);
    if let Ok(meta) = fs::symlink_metadata(&assets_dir) {
        if meta.file_type().is_symlink() || !meta.is_dir() {
            return Err("Export assets path exists and is not a regular directory".into());
        }
    } else {
        fs::create_dir_all(&assets_dir).map_err(|error| error.to_string())?;
    }

    let mut copied: BTreeMap<PathBuf, String> = BTreeMap::new();
    let mut used_names: BTreeSet<String> = BTreeSet::new();
    let encoded_dir = utf8_percent_encode(&assets_dir_name, URL_SEGMENT_ENCODE).to_string();

    let rewritten = markdown_media::rewrite_markdown_media(text, |dest_url| {
        let Some(canonical) = managed_by_url.get(dest_url) else {
            return Ok(None);
        };
        let file_name = if let Some(existing) = copied.get(canonical) {
            existing.clone()
        } else {
            let unique = unique_asset_name(canonical, &mut used_names);
            let target = assets_dir.join(&unique);
            fs::copy(canonical, &target).map_err(|error| error.to_string())?;
            copied.insert(canonical.clone(), unique.clone());
            unique
        };
        let encoded_file = utf8_percent_encode(&file_name, URL_SEGMENT_ENCODE).to_string();
        Ok(Some(format!("./{encoded_dir}/{encoded_file}")))
    })?;

    files::write(app, destination, rewritten.as_bytes())
}

pub(super) fn decode_asset_path(raw: &str) -> Option<PathBuf> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }
    for prefix in [
        "asset://localhost/",
        "http://asset.localhost/",
        "https://asset.localhost/",
    ] {
        if let Some(rest) = trimmed.strip_prefix(prefix) {
            let decoded = percent_decode_str(rest).decode_utf8().ok()?;
            if decoded.is_empty() {
                return None;
            }
            let path = PathBuf::from(decoded.as_ref());
            #[cfg(unix)]
            if !path.is_absolute() {
                return Some(PathBuf::from(format!("/{decoded}")));
            }
            return Some(path);
        }
    }
    if trimmed.contains("://") {
        return None;
    }
    Some(PathBuf::from(trimmed))
}

fn unique_asset_name(path: &Path, used: &mut BTreeSet<String>) -> String {
    let stem = path
        .file_stem()
        .and_then(|part| part.to_str())
        .filter(|part| !part.is_empty())
        .unwrap_or("asset");
    let ext = path
        .extension()
        .and_then(|part| part.to_str())
        .filter(|part| !part.is_empty());
    let base = match ext {
        Some(ext) => format!("{stem}.{ext}"),
        None => stem.to_owned(),
    };
    if used.insert(base.clone()) {
        return base;
    }
    for index in 2..10_000 {
        let candidate = match ext {
            Some(ext) => format!("{stem}-{index}.{ext}"),
            None => format!("{stem}-{index}"),
        };
        if used.insert(candidate.clone()) {
            return candidate;
        }
    }
    base
}
