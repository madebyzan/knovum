pub(crate) mod external_source;
mod json_value;
mod media;
mod object;
mod payload;
mod projection;
mod scalars;
mod validation;
pub(crate) mod visual;

use serde_json::Value;
use sqlx::{Sqlite, Transaction};

use self::payload::validate_payload_shape;

pub(crate) use object::validate_object_status;
pub(in crate::spaces) use projection::{validate_query_filter, validate_same_space_node};

pub(super) async fn validate_node(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    kind: &str,
    node_payload: &Value,
) -> Result<Option<String>, String> {
    payload::validate_base_fields(kind, node_payload)?;
    validate_payload_shape(kind, node_payload)?;
    match kind {
        "object" => object::validate_object(tx, space_id, node_payload)
            .await
            .map(Some),
        "portal" => {
            projection::validate_portal(tx, node_payload).await?;
            Ok(None)
        }
        "note" | "text" | "website" | "markdown" | "image" | "video" | "audio" | "file"
        | "code" | "credential" => Ok(None),
        _ => Err("Unsupported Space node kind".into()),
    }
}
