use serde_json::{Map, Value};

use super::super::scalars::{
    optional_bool, optional_integer_range, optional_safe_nonnegative_integer, optional_string,
    optional_string_allow_empty, required_string, required_string_allow_empty,
};
use super::super::validation::reject_unknown;
use super::{MAX_PREVIEW_TEXT, MAX_TEXT};

pub(super) fn validate_markdown(object: &Map<String, Value>) -> Result<(), String> {
    required_string_allow_empty(object, "text", MAX_TEXT)?;
    if let Some(content) = object.get("tiptapContent") {
        super::tiptap::validate_document(content)?;
    }
    if let Some(state) =
        super::super::scalars::optional_string(object, "derivationState", 16, false)?
    {
        if !matches!(state, "pending" | "failed") {
            return Err("Markdown derivationState is invalid".into());
        }
    }
    Ok(())
}

pub(super) fn validate_asset(object: &Map<String, Value>) -> Result<(), String> {
    required_string(object, "assetPath", 4096)?;
    required_string(object, "fileName", 1024)?;
    optional_string_allow_empty(object, "previewPath", 4096)?;
    optional_string_allow_empty(object, "previewText", MAX_PREVIEW_TEXT)?;
    optional_safe_nonnegative_integer(object, "size")?;
    optional_safe_nonnegative_integer(object, "modifiedAt")?;
    if let Some(state) =
        super::super::scalars::optional_string(object, "processingState", 16, false)?
    {
        if !matches!(state, "importing" | "processing" | "failed") {
            return Err("Space asset processingState is invalid".into());
        }
    }
    Ok(())
}

pub(super) fn validate_visual_asset(object: &Map<String, Value>) -> Result<(), String> {
    validate_asset(object)?;
    optional_integer_range(object, "intrinsicWidth", 1, 100_000)?;
    optional_integer_range(object, "intrinsicHeight", 1, 100_000)?;
    Ok(())
}

pub(super) fn validate_media_metadata(object: &Map<String, Value>) -> Result<(), String> {
    super::super::media::analysis::validate(object.get("mediaAnalysis"))?;
    optional_string(object, "creator", 512, false)?;
    optional_safe_nonnegative_integer(object, "durationSeconds")?;
    let Some(origin) = object.get("origin") else {
        return Ok(());
    };
    let origin = origin
        .as_object()
        .ok_or_else(|| "Media origin must be an object".to_string())?;
    reject_unknown(
        origin,
        ["kind", "url", "provider", "externalId"],
        "Media origin",
    )?;
    if required_string(origin, "kind", 16)? != "web" {
        return Err("Media origin kind must be web".into());
    }
    validate_http_url(origin, "url")?;
    required_string(origin, "provider", 128)?;
    required_string(origin, "externalId", 512)?;
    Ok(())
}

fn validate_http_url(object: &Map<String, Value>, key: &str) -> Result<(), String> {
    let value = required_string(object, key, 4096)?;
    let url = reqwest::Url::parse(value).map_err(|_| "Media origin URL is invalid")?;
    if !matches!(url.scheme(), "http" | "https")
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
    {
        return Err("Media origin URL must use HTTP or HTTPS".into());
    }
    Ok(())
}

pub(super) fn validate_file(object: &Map<String, Value>) -> Result<(), String> {
    validate_asset(object)?;
    optional_string_allow_empty(object, "extractedText", MAX_TEXT)?;
    if let Some(entries) = object.get("entries") {
        validate_entries(entries)?;
    }
    if let Some(coverage) = object.get("documentCoverage") {
        let report =
            crate::spaces::knowledge::model::document_coverage::DocumentCoverage::decode(coverage)?;
        if object.get("sourceRevision").and_then(Value::as_str)
            != Some(report.source_revision.as_str())
        {
            return Err("Document coverage belongs to an obsolete source revision".into());
        }
    }
    Ok(())
}

fn validate_entries(value: &Value) -> Result<(), String> {
    let entries = value
        .as_array()
        .ok_or_else(|| "Resource entries must be an array".to_string())?;
    if entries.len() > 200 {
        return Err("Resource entries exceed the maximum of 200".into());
    }
    for entry in entries {
        let entry = entry
            .as_object()
            .ok_or_else(|| "Resource entry must be an object".to_string())?;
        reject_unknown(entry, ["path", "kind", "size"], "Resource entry")?;
        required_string(entry, "path", 4096)?;
        let kind = required_string(entry, "kind", 16)?;
        if !matches!(kind, "file" | "folder") {
            return Err("Resource entry kind is invalid".into());
        }
        optional_safe_nonnegative_integer(entry, "size")?;
    }
    Ok(())
}

pub(super) fn validate_code(object: &Map<String, Value>) -> Result<(), String> {
    validate_asset(object)?;
    required_string(object, "previewLanguage", 64)?;
    Ok(())
}

pub(super) fn validate_credential(object: &Map<String, Value>) -> Result<(), String> {
    let kind = required_string(object, "credentialKind", 32)?;
    if !matches!(kind, "api-key" | "account-password") {
        return Err("Credential kind is invalid".into());
    }
    optional_bool(object, "credentialConfigured")
}
