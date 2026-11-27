use std::{fs, path::Path};

use super::super::{kind::AssetKind, PreparedAsset};

const MAX_MARKDOWN_BYTES: u64 = 4 * 1024 * 1024;

pub(super) fn is_markdown(path: &Path) -> bool {
    matches!(
        path.extension()
            .and_then(|value| value.to_str())
            .map(str::to_ascii_lowercase)
            .as_deref(),
        Some("md" | "markdown")
    )
}

pub(super) fn import_markdown(
    app: &tauri::AppHandle,
    source: &Path,
    name: String,
    boundary_root: Option<&Path>,
    claim_embedded: &mut dyn FnMut(&Path) -> Result<(), String>,
) -> Result<PreparedAsset, String> {
    let metadata = fs::metadata(source).map_err(|error| error.to_string())?;
    if metadata.len() > MAX_MARKDOWN_BYTES {
        return Err("Markdown files must be smaller than 4 MiB".into());
    }
    let raw = fs::read_to_string(source).map_err(|_| "Markdown must be valid UTF-8".to_owned())?;
    let text = match source.parent() {
        Some(base_dir) => super::super::markdown_media::materialize_local_media(
            app,
            &raw,
            base_dir,
            boundary_root.unwrap_or(base_dir),
            claim_embedded,
        )?,
        None => raw,
    };
    Ok(PreparedAsset {
        name,
        path: None,
        kind: AssetKind::Markdown,
        text: Some(text),
        preview_text: None,
        preview_language: None,
        preview_path: None,
        extracted_text: None,
        size: None,
        modified_at: None,
        source_revision: None,
        processing_state: None,
        intrinsic_width: None,
        intrinsic_height: None,
        knowledge_chunks: None,
        document_coverage: None,
        entries: None,
    })
}
