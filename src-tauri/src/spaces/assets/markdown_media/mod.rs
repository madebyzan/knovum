use std::{
    collections::BTreeMap,
    fs,
    ops::Range,
    path::{Component, Path, PathBuf},
};

use percent_encoding::percent_decode_str;
use pulldown_cmark::{Event, Options, Parser, Tag};
use uuid::Uuid;

use super::{asset_directory, image, references::to_webview_url};

pub(crate) const MAX_EDITOR_VIDEO_BYTES: usize = 256 * 1024 * 1024;

const MARKDOWN_MEDIA_OPTIONS: Options = Options::ENABLE_TABLES
    .union(Options::ENABLE_STRIKETHROUGH)
    .union(Options::ENABLE_TASKLISTS)
    .union(Options::ENABLE_MATH);

/// Returns true when `path` carries a web-playable video container extension.
pub(crate) fn is_video_path(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| {
            matches!(
                ext.to_ascii_lowercase().as_str(),
                "mp4" | "webm" | "mov" | "m4v"
            )
        })
}

/// Validates container magic bytes and size bounds for a Markdown-embedded video file.
pub(crate) fn detect_video_extension(bytes: &[u8]) -> Result<&'static str, String> {
    if bytes.is_empty() || bytes.len() > MAX_EDITOR_VIDEO_BYTES {
        return Err("Editor videos must be between 1 byte and 256 MiB".to_owned());
    }
    if bytes.len() >= 12 && &bytes[4..8] == b"ftyp" {
        return Ok(if &bytes[8..12] == b"qt  " {
            "mov"
        } else {
            "mp4"
        });
    }
    if bytes.len() >= 4 && bytes[0..4] == [0x1A, 0x45, 0xDF, 0xA3] {
        return Ok("webm");
    }
    Err("Unsupported editor video format".to_owned())
}

/// Persists validated video bytes as a managed Space editor asset.
pub(crate) fn persist_detected_video_bytes(
    app: &tauri::AppHandle,
    bytes: Vec<u8>,
) -> Result<String, String> {
    let extension = detect_video_extension(&bytes)?;
    let target = asset_directory(app)?.join(format!("editor-{}.{extension}", Uuid::new_v4()));
    fs::write(&target, bytes).map_err(|error| error.to_string())?;
    Ok(target.to_string_lossy().into_owned())
}

/// Imports a trusted local image or video file into managed Space editor assets.
pub(crate) fn import_trusted_media_file(
    app: &tauri::AppHandle,
    source: &Path,
) -> Result<String, String> {
    if !is_video_path(source) {
        return image::import_trusted_file(app, source);
    }
    if !source.is_file() {
        return Err("Editor videos must be regular files".into());
    }
    let size = fs::metadata(source)
        .map_err(|error| error.to_string())?
        .len();
    let size = usize::try_from(size).map_err(|_| "Editor video size exceeds this platform")?;
    if size == 0 || size > MAX_EDITOR_VIDEO_BYTES {
        return Err("Editor videos must be between 1 byte and 256 MiB".to_owned());
    }
    let bytes = fs::read(source).map_err(|error| error.to_string())?;
    persist_detected_video_bytes(app, bytes)
}

/// Rewrites Markdown media (`![alt](dest)`) destination URLs outside code blocks/spans using
/// `resolve`. Returning `Ok(Some(replacement))` replaces the destination slice; `Ok(None)` keeps
/// it intact.
pub(crate) fn rewrite_markdown_media<F>(markdown: &str, mut resolve: F) -> Result<String, String>
where
    F: FnMut(&str) -> Result<Option<String>, String>,
{
    let mut replacements: Vec<(Range<usize>, String)> = Vec::new();
    for (event, range) in Parser::new_ext(markdown, MARKDOWN_MEDIA_OPTIONS).into_offset_iter() {
        let Event::Start(Tag::Image { dest_url, .. }) = event else {
            continue;
        };
        let slice = &markdown[range.start..range.end];
        let Some(local_range) = locate_media_url_range(slice, &dest_url) else {
            continue;
        };
        if let Some(replacement) = resolve(&dest_url)? {
            replacements.push((
                (range.start + local_range.start)..(range.start + local_range.end),
                replacement,
            ));
        }
    }
    if replacements.is_empty() {
        return Ok(markdown.to_owned());
    }
    Ok(apply_replacements(markdown, replacements))
}

/// Imports local image and video files referenced inside Markdown (relative to `base_dir` and
/// strictly confined within `boundary_root`) into Knovum's managed Space assets and rewrites
/// their URLs to canonical WebView asset URLs.
pub(crate) fn materialize_local_media(
    app: &tauri::AppHandle,
    markdown: &str,
    base_dir: &Path,
    boundary_root: &Path,
    claim_embedded: &mut dyn FnMut(&Path) -> Result<(), String>,
) -> Result<String, String> {
    let Ok(canonical_root) = boundary_root.canonicalize() else {
        return Ok(markdown.to_owned());
    };
    let mut imported: BTreeMap<PathBuf, String> = BTreeMap::new();
    rewrite_markdown_media(markdown, |dest_url| {
        let Some(candidate) = resolve_local_candidate(dest_url, base_dir, &canonical_root) else {
            return Ok(None);
        };
        if let Some(existing_url) = imported.get(&candidate) {
            return Ok(Some(existing_url.clone()));
        }
        let managed = match import_trusted_media_file(app, &candidate) {
            Ok(path) => path,
            Err(error) => {
                log::warn!(
                    "Skipping invalid local Markdown media {}: {error}",
                    candidate.display()
                );
                return Ok(None);
            }
        };
        claim_embedded(Path::new(&managed))?;
        let url = to_webview_url(Path::new(&managed))?;
        imported.insert(candidate, url.clone());
        Ok(Some(url))
    })
}

fn resolve_local_candidate(
    dest_url: &str,
    base_dir: &Path,
    canonical_root: &Path,
) -> Option<PathBuf> {
    let trimmed = dest_url.trim();
    if trimmed.is_empty() || trimmed.starts_with('#') || has_uri_scheme(trimmed) {
        return None;
    }
    let without_suffix = trimmed.split(['?', '#']).next().unwrap_or(trimmed);
    let decoded = percent_decode_str(without_suffix).decode_utf8().ok()?;
    let rel = Path::new(decoded.as_ref());
    if rel
        .components()
        .any(|component| matches!(component, Component::Prefix(_)))
    {
        return None;
    }
    let joined = if rel.is_absolute() {
        rel.to_path_buf()
    } else {
        base_dir.join(rel)
    };
    let meta = fs::symlink_metadata(&joined).ok()?;
    if meta.file_type().is_symlink() || !meta.is_file() {
        return None;
    }
    let canonical = joined.canonicalize().ok()?;
    if !canonical.starts_with(canonical_root) || !canonical.is_file() {
        return None;
    }
    Some(canonical)
}

fn has_uri_scheme(value: &str) -> bool {
    if value.starts_with("//") {
        return true;
    }
    let Some((scheme, _)) = value.split_once(':') else {
        return false;
    };
    !scheme.is_empty()
        && scheme.len() > 1
        && scheme.bytes().enumerate().all(|(idx, b)| match idx {
            0 => b.is_ascii_alphabetic(),
            _ => b.is_ascii_alphanumeric() || matches!(b, b'+' | b'-' | b'.'),
        })
}

fn locate_media_url_range(slice: &str, dest_url: &str) -> Option<Range<usize>> {
    let paren = slice.find("](")?;
    let after_paren = paren + 2;
    let rest = &slice[after_paren..];
    let trimmed_start = rest.len() - rest.trim_start().len();
    let dest_start = after_paren + trimmed_start;
    let dest_slice = &slice[dest_start..];
    if let Some(angle) = dest_slice.strip_prefix('<') {
        let close = angle.find('>')?;
        return Some(dest_start..(dest_start + close + 2));
    }
    if dest_slice.starts_with(dest_url) {
        return Some(dest_start..(dest_start + dest_url.len()));
    }
    let end = dest_slice.find(|c: char| c.is_whitespace() || c == ')')?;
    (end > 0).then_some(dest_start..(dest_start + end))
}

fn apply_replacements(text: &str, mut replacements: Vec<(Range<usize>, String)>) -> String {
    replacements.sort_by_key(|(range, _)| range.start);
    let mut output = String::with_capacity(text.len());
    let mut cursor = 0;
    for (range, replacement) in replacements {
        if range.start < cursor || range.end > text.len() {
            continue;
        }
        output.push_str(&text[cursor..range.start]);
        output.push_str(&replacement);
        cursor = range.end;
    }
    output.push_str(&text[cursor..]);
    output
}

#[cfg(test)]
mod tests;
