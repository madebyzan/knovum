pub(crate) mod nodes;

use serde_json::Value;
use sqlx::{Sqlite, Transaction};

use super::model::revision;

pub(crate) use nodes::validate_object_status;
pub(in crate::spaces) use nodes::{validate_query_filter, validate_same_space_node};

pub(crate) async fn validate_node(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    kind: &str,
    data: &Value,
) -> Result<Option<String>, String> {
    let object_type_id = nodes::validate_node(tx, space_id, kind, data).await?;
    revision::validate_declared(data)?;
    Ok(object_type_id)
}

pub(crate) async fn normalize_node(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    kind: &str,
    data: &mut Value,
) -> Result<Option<String>, String> {
    let object_type_id = nodes::validate_node(tx, space_id, kind, data).await?;
    revision::apply(data)?;
    Ok(object_type_id)
}
