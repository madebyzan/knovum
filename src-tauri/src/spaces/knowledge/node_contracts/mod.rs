pub(crate) mod document_coverage;
mod json_value;
mod media;
mod object;
mod payload;
mod projection;
mod scalars;
mod validation;
mod visual;

use serde_json::Value;
use sqlx::{Sqlite, Transaction};

use self::payload::validate_payload_shape;

pub(super) async fn validate_node(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    kind: &str,
    data: &Value,
) -> Result<Option<String>, String> {
    payload::validate_base_fields(kind, data)?;
    validate_payload_shape(kind, data)?;
    match kind {
        "object" => object::validate_object(tx, space_id, data).await.map(Some),
        "portal" => {
            projection::validate_portal(tx, data).await?;
            Ok(None)
        }
        "note" | "text" | "website" | "markdown" | "image" | "video" | "audio" | "file"
        | "code" | "credential" => Ok(None),
        _ => Err("Unsupported Space node kind".into()),
    }
}

pub(super) async fn validate_same_space_node(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    node_id: &str,
    target_id: &str,
) -> Result<(), String> {
    projection::validate_same_space_node(tx, space_id, node_id, target_id).await
}

pub(super) async fn validate_query_filter(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    value: &Value,
) -> Result<(), String> {
    projection::validate_query_filter(tx, space_id, value).await
}

pub(super) fn validate_object_status(status: &str) -> Result<(), String> {
    object::validate_object_status(status)
}
