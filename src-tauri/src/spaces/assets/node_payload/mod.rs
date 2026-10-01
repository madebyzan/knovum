use serde_json::{json, Value};

use super::PreparedAsset;

pub(crate) fn from_import(asset: PreparedAsset) -> Result<Value, String> {
    if asset.kind == "markdown" {
        let text = asset.text.ok_or("Imported Markdown has no text")?;
        let (title, text) = markdown_document(&asset.name, &text);
        return Ok(
            json!({"kind":"markdown", "title":title, "text":text, "sourceRevision":asset.source_revision}),
        );
    }
    let path = asset
        .path
        .as_ref()
        .ok_or("Imported resource has no managed path")?;
    let document_coverage = asset.document_coverage.clone();
    let mut data =
        json!({"kind":asset.kind, "title":asset.name, "fileName":asset.name, "assetPath":path});
    let encoded = serde_json::to_value(&asset).map_err(|error| error.to_string())?;
    for field in [
        "sourceRevision",
        "previewPath",
        "previewText",
        "size",
        "modifiedAt",
        "processingState",
        "intrinsicWidth",
        "intrinsicHeight",
        "extractedText",
        "entries",
    ] {
        if let Some(value) = encoded.get(field) {
            data[field] = value.clone();
        }
    }
    if let Some(coverage) = document_coverage {
        data["documentCoverage"] = coverage;
    }
    if asset.kind == "code" {
        data["previewLanguage"] = json!(asset
            .preview_language
            .ok_or("Imported code has no language")?);
    }
    Ok(data)
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
