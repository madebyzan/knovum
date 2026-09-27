use crate::spaces::db;
use sqlx::{Row, Sqlite, Transaction};
use std::collections::HashSet;

pub(in crate::spaces::structure) async fn reject_object_dependents(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    node_ids: &HashSet<String>,
) -> Result<(), String> {
    if node_ids.is_empty() {
        return Ok(());
    }
    let rows = sqlx::query(
        "SELECT n.id,n.data,t.fields FROM spaces_nodes n \
         LEFT JOIN spaces_object_types t ON t.id=n.object_type_id \
         WHERE n.space_id=$1 AND n.kind='object' AND n.deleted_at IS NULL",
    )
    .bind(space_id)
    .fetch_all(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    for row in rows {
        let id: String = row.try_get("id").map_err(db::sql_error)?;
        if node_ids.contains(&id) {
            continue;
        }
        let raw: String = row.try_get("data").map_err(db::sql_error)?;
        let fields: Option<String> = row.try_get("fields").map_err(db::sql_error)?;
        let data = serde_json::from_str(&raw).map_err(db::json_error)?;
        if crate::spaces::knowledge::object_references::collect_node_ids(fields.as_deref(), &data)?
            .iter()
            .any(|target| node_ids.contains(target))
        {
            return Err(format!(
                "Retained object {id} still references permanently deleted content"
            ));
        }
    }
    Ok(())
}
