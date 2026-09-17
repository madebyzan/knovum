mod locations;
mod office_geometry;
mod values;

use super::contracts::validate_text_limit;
use values::{ensure_keys, optional_string_limited, required_string};

use serde_json::Value;

const MAX_REVISION_CHARS: usize = 2_048;
const MAX_QUOTE_EXACT_CHARS: usize = 64_000;
const MAX_CONTEXT_CHARS: usize = 8_000;
const MAX_LOCATION_TEXT_CHARS: usize = 16_000;
const MAX_HEADING_PATH_ITEMS: usize = 256;
const MAX_PDF_RECTS: usize = 512;

pub fn validate(locator: &Value, expected_revision: Option<&str>) -> Result<(), String> {
    let object = locator
        .as_object()
        .ok_or_else(|| "Knowledge locator must be an object".to_string())?;
    ensure_keys(
        object,
        &["sourceRevision", "location", "quote"],
        "Knowledge locator",
    )?;
    let revision = required_string(object, "sourceRevision", "Knowledge locator sourceRevision")?;
    validate_text_limit(
        revision,
        MAX_REVISION_CHARS,
        "Knowledge locator sourceRevision",
    )?;
    if expected_revision.is_some_and(|expected| expected != revision) {
        return Err(
            "Knowledge locator sourceRevision must equal the stored source revision".into(),
        );
    }
    locations::validate_location(
        object
            .get("location")
            .ok_or_else(|| "Knowledge locator location is required".to_string())?,
    )?;
    if let Some(quote) = object.get("quote") {
        validate_quote(quote)?;
    }
    Ok(())
}

fn validate_quote(value: &Value) -> Result<(), String> {
    let object = value
        .as_object()
        .ok_or_else(|| "Knowledge locator quote must be an object".to_string())?;
    ensure_keys(
        object,
        &["exact", "prefix", "suffix"],
        "Knowledge locator quote",
    )?;
    let exact = required_string(object, "exact", "Knowledge locator quote exact")?;
    validate_text_limit(
        exact,
        MAX_QUOTE_EXACT_CHARS,
        "Knowledge locator quote exact",
    )?;
    optional_string_limited(
        object,
        "prefix",
        "Knowledge locator quote prefix",
        MAX_CONTEXT_CHARS,
    )?;
    optional_string_limited(
        object,
        "suffix",
        "Knowledge locator quote suffix",
        MAX_CONTEXT_CHARS,
    )?;
    Ok(())
}
