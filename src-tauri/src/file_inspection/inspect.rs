use std::{fs, path::Path};

use super::{
    classification::{classify_file, ResourceClassification},
    text_preview::{modified_millis, read_html_preview, read_text_preview},
    InspectedFile,
};

#[derive(Default)]
struct ResourcePreview {
    text: Option<String>,
    language: Option<String>,
}

pub fn inspect_file(path: &Path, logical_name: &str) -> Result<InspectedFile, String> {
    let metadata = fs::symlink_metadata(path).map_err(|error| error.to_string())?;
    reject_unsupported(&metadata)?;
    inspect_regular_file(path, logical_name, &metadata)
}

fn inspect_regular_file(
    path: &Path,
    logical_name: &str,
    metadata: &fs::Metadata,
) -> Result<InspectedFile, String> {
    let logical_path = Path::new(logical_name);
    let classification = classify_file(logical_path);
    let (preview, entries) = match classification {
        ResourceClassification::SourceCode(language) => {
            let text = if language == "html" {
                read_html_preview(path)?
            } else {
                read_text_preview(path)?
            };
            (
                ResourcePreview {
                    text: Some(text),
                    language: Some(language.to_owned()),
                },
                None,
            )
        }
        ResourceClassification::TextDocument => (
            ResourcePreview {
                text: Some(read_text_preview(path)?),
                language: None,
            },
            None,
        ),
        ResourceClassification::Archive => {
            let entries = super::archive_entries(path, logical_name)?;
            let text = Some(super::entry_search_text(&entries));
            (
                ResourcePreview {
                    text,
                    language: None,
                },
                Some(entries),
            )
        }
        _ => (ResourcePreview::default(), None),
    };
    Ok(InspectedFile {
        name: logical_name.to_owned(),
        path: path.to_string_lossy().into_owned(),
        kind: classification.node_kind(),
        preview_text: preview.text,
        preview_language: preview.language,
        size: metadata.len(),
        modified_at: modified_millis(metadata)?,
        entries,
    })
}

fn reject_unsupported(metadata: &fs::Metadata) -> Result<(), String> {
    if metadata.file_type().is_symlink() {
        return Err("Symbolic links are not supported as Space resources".into());
    }
    if !metadata.is_file() {
        return Err("Only regular files can be imported into Spaces".into());
    }
    Ok(())
}

#[cfg(test)]
#[path = "tests/inspect.rs"]
mod tests;
