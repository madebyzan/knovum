use serde_json::Value;

use super::super::{MAX_CITATIONS, MAX_CITATION_QUOTE_CHARS, MAX_CITATION_TITLE_CHARS};
use super::CurrentCitationRef;
use crate::spaces::knowledge::locator;

pub(super) fn citation_shapes(citations: &Value) -> Result<Vec<CurrentCitationRef>, String> {
    let citations = citations
        .as_array()
        .ok_or_else(|| "Grounded AI message citations must be an array".to_string())?;
    if citations.len() > MAX_CITATIONS {
        return Err(format!(
            "Grounded AI message exceeds {MAX_CITATIONS} citations"
        ));
    }
    let mut live = Vec::new();
    for citation in citations {
        if let Some(reference) = citation_shape(citation)? {
            live.push(reference);
        }
    }
    Ok(live)
}

fn citation_shape(citation: &Value) -> Result<Option<CurrentCitationRef>, String> {
    let object = citation
        .as_object()
        .ok_or_else(|| "Grounded AI citation must be an object".to_string())?;
    let allowed = [
        "index",
        "state",
        "sourceSpaceTitle",
        "chunkId",
        "nodeId",
        "nodeTitle",
        "sourceRevision",
        "locator",
        "quote",
    ];
    if object.keys().any(|key| !allowed.contains(&key.as_str())) {
        return Err("Grounded AI citation contains an unsupported property".into());
    }
    if let Some(index_val) = citation.get("index") {
        let index_num = index_val
            .as_u64()
            .ok_or_else(|| "Grounded AI citation index must be a positive integer".to_string())?;
        if index_num == 0 {
            return Err("Grounded AI citation index must be positive".to_string());
        }
    }
    let state = required_text(citation, "state")?;
    let revision = required_text(citation, "sourceRevision")?;
    validate_snapshot_text(citation)?;
    let locator_value = citation
        .get("locator")
        .ok_or_else(|| "Grounded AI citation locator is required".to_string())?;
    locator::validate(locator_value, Some(revision))?;
    match state {
        "current" => Ok(Some(CurrentCitationRef {
            chunk_id: required_text(citation, "chunkId")?.to_owned(),
            node_id: required_text(citation, "nodeId")?.to_owned(),
            revision: revision.to_owned(),
        })),
        "detached" => {
            if object.contains_key("chunkId") || object.contains_key("nodeId") {
                return Err(
                    "Detached Grounded AI citations must not carry live node or chunk IDs".into(),
                );
            }
            Ok(None)
        }
        _ => Err("Unsupported Grounded AI citation state".into()),
    }
}

fn validate_snapshot_text(citation: &Value) -> Result<(), String> {
    validate_citation_text(
        required_text(citation, "nodeTitle")?,
        MAX_CITATION_TITLE_CHARS,
        "Grounded AI citation title",
    )?;
    validate_citation_text(
        required_text(citation, "quote")?,
        MAX_CITATION_QUOTE_CHARS,
        "Grounded AI citation quote",
    )?;
    if let Some(space_title) = citation.get("sourceSpaceTitle").and_then(Value::as_str) {
        validate_citation_text(
            space_title,
            MAX_CITATION_TITLE_CHARS,
            "Grounded AI citation Space title",
        )?;
    }
    Ok(())
}

fn validate_citation_text(value: &str, max_chars: usize, label: &str) -> Result<(), String> {
    if value.encode_utf16().count() > max_chars {
        return Err(format!(
            "{label} exceeds the maximum of {max_chars} characters"
        ));
    }
    Ok(())
}

fn required_text<'a>(value: &'a Value, key: &str) -> Result<&'a str, String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .filter(|text| !text.trim().is_empty())
        .ok_or_else(|| format!("Grounded AI citation {key} is required"))
}
