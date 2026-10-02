use std::collections::HashSet;

use serde_json::Value;
use sqlx::{Sqlite, Transaction};

use super::super::object::validate_object_status;
use crate::spaces::{db, knowledge::contracts};

pub(super) async fn validate_query_filter(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    value: &Value,
) -> Result<(), String> {
    let filter = value
        .as_object()
        .ok_or_else(|| "Saved View query filter must be an object".to_string())?;
    let allowed = [
        "text",
        "tags",
        "objectTypeIds",
        "statuses",
        "relationKinds",
        "relationStatuses",
    ];
    if let Some(key) = filter.keys().find(|key| !allowed.contains(&key.as_str())) {
        return Err(format!(
            "Saved View query filter field '{key}' is not supported"
        ));
    }
    if let Some(value) = filter.get("text") {
        validate_limited_string(value, "Saved View query text", 512)?;
    }
    validate_string_array(filter.get("tags"), "Saved View tags", 64, 80)?;
    if let Some(ids) = validate_string_array(
        filter.get("objectTypeIds"),
        "Saved View object types",
        64,
        128,
    )? {
        validate_object_type_ids(tx, space_id, &ids).await?;
    }
    if let Some(statuses) =
        validate_string_array(filter.get("statuses"), "Saved View statuses", 32, 32)?
    {
        for status in statuses {
            validate_object_status(status)?;
        }
    }
    if let Some(kinds) = validate_string_array(
        filter.get("relationKinds"),
        "Saved View relation kinds",
        32,
        32,
    )? {
        for kind in kinds {
            contracts::validate_relation_kind(kind)?;
        }
    }
    if let Some(statuses) = validate_string_array(
        filter.get("relationStatuses"),
        "Saved View relation statuses",
        8,
        32,
    )? {
        for status in statuses {
            contracts::validate_relation_status(status)?;
        }
    }
    Ok(())
}

async fn validate_object_type_ids(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    ids: &[&str],
) -> Result<(), String> {
    if ids.is_empty() {
        return Ok(());
    }
    let unique = ids.iter().copied().collect::<HashSet<_>>();
    if unique.len() != ids.len() {
        return Err("Saved View object types must not contain duplicates".into());
    }
    let json = serde_json::to_string(ids).map_err(db::json_error)?;
    let count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM spaces_object_types WHERE (system=1 OR space_id=$1) AND id IN (SELECT value FROM json_each($2))",
    )
    .bind(space_id)
    .bind(json)
    .fetch_one(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    if count != ids.len() as i64 {
        return Err(
            "Saved View references an object type that does not exist in this Space".into(),
        );
    }
    Ok(())
}

fn validate_string_array<'a>(
    value: Option<&'a Value>,
    label: &str,
    max_entries: usize,
    max_chars: usize,
) -> Result<Option<Vec<&'a str>>, String> {
    let Some(value) = value else {
        return Ok(None);
    };
    let values = value
        .as_array()
        .ok_or_else(|| format!("{label} must be an array"))?;
    if values.len() > max_entries {
        return Err(format!("{label} exceeds the maximum of {max_entries}"));
    }
    values
        .iter()
        .map(|candidate| {
            let text = candidate
                .as_str()
                .filter(|text| !text.is_empty())
                .ok_or_else(|| format!("{label} must contain non-empty strings"))?;
            if text.encode_utf16().count() > max_chars {
                return Err(format!(
                    "{label} entries may contain at most {max_chars} characters"
                ));
            }
            Ok(text)
        })
        .collect::<Result<Vec<_>, _>>()
        .map(Some)
}

fn validate_limited_string(value: &Value, label: &str, max_chars: usize) -> Result<(), String> {
    let text = value
        .as_str()
        .ok_or_else(|| format!("{label} must be a string"))?;
    if text.encode_utf16().count() > max_chars {
        return Err(format!("{label} exceeds {max_chars} characters"));
    }
    Ok(())
}
