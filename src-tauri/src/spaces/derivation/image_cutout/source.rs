use serde_json::Value;
use sqlx::{Sqlite, Transaction};

use crate::spaces::db;

pub(super) async fn revalidate(
    tx: &mut Transaction<'_, Sqlite>,
    derivation_id: &str,
    space_id: &str,
    source_id: &str,
    source_edit: &Value,
) -> Result<(), String> {
    let data: Option<String> = sqlx::query_scalar(
        "SELECT n.data FROM spaces_knowledge_derivations d JOIN spaces_knowledge_derivation_inputs i ON i.derivation_id=d.id AND i.position=0 AND i.role='primary' JOIN spaces_nodes n ON n.id=i.node_id WHERE d.id=$1 AND n.id=$2 AND n.space_id=$3 AND n.deleted_at IS NULL AND n.kind='image' AND json_extract(n.data,'$.sourceRevision')=i.source_revision",
    )
    .bind(derivation_id)
    .bind(source_id)
    .bind(space_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    let data = data.ok_or("Image source changed during background removal")?;
    let data: Value = serde_json::from_str(&data).map_err(db::json_error)?;
    if data.get("imageEdit").unwrap_or(&Value::Null) != source_edit {
        return Err("Image source changed during background removal".into());
    }
    Ok(())
}
