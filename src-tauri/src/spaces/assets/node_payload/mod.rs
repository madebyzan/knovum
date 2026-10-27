use serde::Serialize;
use serde_json::Value;

use super::{
    kind::{AssetKind, PreparedAssetState},
    PreparedAsset,
};
use crate::file_inspection::ArchiveMember;

/// The node a Markdown import becomes: a Knovum document, not a file.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct MarkdownNode<'a> {
    kind: AssetKind,
    title: String,
    text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    source_revision: Option<&'a str>,
}

/// The node any other import becomes: the managed file and what was learned about it. A field is
/// a value or absent, never `null`, which the node schema rejects.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ResourceNode<'a> {
    kind: AssetKind,
    title: &'a str,
    file_name: &'a str,
    asset_path: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    source_revision: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    preview_path: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    preview_text: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    preview_language: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    size: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    modified_at: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    processing_state: Option<PreparedAssetState>,
    #[serde(skip_serializing_if = "Option::is_none")]
    intrinsic_width: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    intrinsic_height: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    extracted_text: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    entries: Option<&'a [ArchiveMember]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    document_coverage: Option<&'a Value>,
}

pub(crate) fn from_import(asset: PreparedAsset) -> Result<Value, String> {
    let data = if asset.kind == AssetKind::Markdown {
        let text = asset
            .text
            .as_deref()
            .ok_or("Imported Markdown has no text")?;
        let (title, text) = markdown_document(&asset.name, text);
        serde_json::to_value(MarkdownNode {
            kind: asset.kind,
            title,
            text,
            source_revision: asset.source_revision.as_deref(),
        })
    } else {
        serde_json::to_value(resource_node(&asset)?)
    };
    data.map_err(|error| error.to_string())
}

fn resource_node(asset: &PreparedAsset) -> Result<ResourceNode<'_>, String> {
    let preview_language = match asset.kind {
        AssetKind::Code => Some(
            asset
                .preview_language
                .as_deref()
                .ok_or("Imported code has no language")?,
        ),
        _ => None,
    };
    Ok(ResourceNode {
        kind: asset.kind,
        title: &asset.name,
        file_name: &asset.name,
        asset_path: asset
            .path
            .as_deref()
            .ok_or("Imported resource has no managed path")?,
        source_revision: asset.source_revision.as_deref(),
        preview_path: asset.preview_path.as_deref(),
        preview_text: asset.preview_text.as_deref(),
        preview_language,
        size: asset.size,
        modified_at: asset.modified_at,
        processing_state: asset.processing_state,
        intrinsic_width: asset.intrinsic_width,
        intrinsic_height: asset.intrinsic_height,
        extracted_text: asset.extracted_text.as_deref(),
        entries: asset.entries.as_deref(),
        document_coverage: asset.document_coverage.as_ref(),
    })
}

fn markdown_document(name: &str, raw: &str) -> (String, String) {
    let first = raw
        .trim_start()
        .split_once('\n')
        .map_or(raw.trim_start(), |(line, _)| line)
        .trim_end_matches('\r');
    if let Some(heading) = first
        .strip_prefix('#')
        .filter(|text| text.starts_with([' ', '\t']))
    {
        let heading = heading.trim();
        let hashes = heading.trim_end_matches('#');
        let title = if hashes.len() < heading.len() && hashes.ends_with(char::is_whitespace) {
            hashes.trim_end()
        } else {
            heading
        };
        if !title.is_empty() {
            let consumed = raw.len() - raw.trim_start().len() + first.len();
            let text = raw[consumed..].trim_start_matches(['\r', '\n']);
            return (truncate_title(title), text.to_owned());
        }
    }
    let lowercase = name.to_ascii_lowercase();
    let title = [".markdown", ".md"]
        .iter()
        .find(|suffix| lowercase.ends_with(**suffix))
        .map_or(name, |suffix| &name[..name.len() - suffix.len()]);
    (
        truncate_title(if title.is_empty() { name } else { title }),
        raw.to_owned(),
    )
}

fn truncate_title(value: &str) -> String {
    value
        .chars()
        .scan(0, |units, character| {
            *units += character.len_utf16();
            (*units <= 512).then_some(character)
        })
        .collect()
}

#[cfg(test)]
mod tests;
