use serde_json::{Map, Value};

use super::super::{
    scalars::{optional_string, optional_string_allow_empty, required_string},
    validation::reject_unknown,
};

pub(super) const FIELDS: &[&str] = &[
    "url",
    "pageTitle",
    "canonicalUrl",
    "siteName",
    "description",
    "contentPreview",
    "websiteSections",
    "faviconPath",
    "previewImagePath",
    "metadataState",
];

pub(super) fn validate(object: &Map<String, Value>) -> Result<(), String> {
    validate_url(required_string(object, "url", 4096)?, "Website URL")?;
    optional_string_allow_empty(object, "pageTitle", 512)?;
    if let Some(url) = optional_string(object, "canonicalUrl", 4096, false)? {
        validate_url(url, "Canonical URL")?;
    }
    optional_string_allow_empty(object, "siteName", 256)?;
    optional_string_allow_empty(object, "description", 4096)?;
    optional_string_allow_empty(object, "contentPreview", 12_000)?;
    optional_string_allow_empty(object, "faviconPath", 4096)?;
    optional_string_allow_empty(object, "previewImagePath", 4096)?;
    validate_metadata_state(object)?;
    validate_sections(object)
}

fn validate_metadata_state(object: &Map<String, Value>) -> Result<(), String> {
    if let Some(state) = optional_string(object, "metadataState", 16, false)? {
        if !matches!(state, "idle" | "loading" | "ready" | "failed") {
            return Err("Website metadataState is invalid".into());
        }
    }
    Ok(())
}

fn validate_sections(object: &Map<String, Value>) -> Result<(), String> {
    let Some(sections) = object.get("websiteSections") else {
        return Ok(());
    };
    let sections = sections
        .as_array()
        .ok_or_else(|| "websiteSections must be an array".to_string())?;
    if sections.len() > 64 {
        return Err("websiteSections exceeds the maximum of 64".into());
    }
    for section in sections {
        let section = section
            .as_object()
            .ok_or_else(|| "Website section must be an object".to_string())?;
        reject_unknown(section, ["heading", "text"], "Website section")?;
        if let Some(heading) = section.get("heading") {
            let heading = heading
                .as_str()
                .ok_or_else(|| "Website section heading must be a string".to_string())?;
            if heading.encode_utf16().count() > 512 {
                return Err("Website section heading is too long".into());
            }
        }
        required_string(section, "text", 12_000)?;
    }
    Ok(())
}

fn validate_url(value: &str, label: &str) -> Result<(), String> {
    let url = reqwest::Url::parse(value).map_err(|_| format!("{label} is invalid"))?;
    if !matches!(url.scheme(), "http" | "https") {
        return Err(format!("{label} must use HTTP or HTTPS"));
    }
    Ok(())
}
