use serde_json::Value;
use sqlx::{Sqlite, Transaction};

use crate::spaces::db;

pub(super) async fn validate_source(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    node_id: &str,
    data: &Value,
) -> Result<(), String> {
    let source_id = data
        .get("sourceVideoNodeId")
        .and_then(Value::as_str)
        .ok_or("Subtitle source video is required")?;
    data.get("sourceVideoRevision")
        .and_then(Value::as_str)
        .ok_or("Subtitle source revision is required")?;
    if source_id == node_id {
        return Err("Subtitle source must differ from the subtitle node".into());
    }
    let exists: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM spaces_nodes WHERE id=$1 AND space_id=$2 AND kind='video'",
    )
    .bind(source_id)
    .bind(space_id)
    .fetch_one(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    if exists == 1 {
        Ok(())
    } else {
        Err("Subtitle source video is unavailable in this Space".into())
    }
}
