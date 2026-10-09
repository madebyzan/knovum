use sqlx::{Sqlite, Transaction};

use crate::spaces::db;

pub(crate) async fn invalidate_for_nodes(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    node_ids: &[String],
) -> Result<(), String> {
    let groups: Vec<String> = sqlx::query_scalar(
        "SELECT DISTINCT history_group_id FROM spaces_edit_history h \
         WHERE h.space_id=$1 AND h.history_group_id IS NOT NULL AND \
         (EXISTS(SELECT 1 FROM json_each(h.before_node_ids) b JOIN json_each($2) n ON n.value=b.value) \
          OR EXISTS(SELECT 1 FROM json_each(h.after_node_ids) a JOIN json_each($2) n ON n.value=a.value))",
    )
    .bind(space_id)
    .bind(sqlx::types::Json(node_ids))
    .fetch_all(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    for group in groups {
        sqlx::query("DELETE FROM spaces_edit_history WHERE history_group_id=$1")
            .bind(group)
            .execute(&mut **tx)
            .await
            .map_err(db::sql_error)?;
    }
    sqlx::query(
        "DELETE FROM spaces_edit_history AS h WHERE h.space_id=$1 AND \
         (EXISTS(SELECT 1 FROM json_each(h.before_node_ids) b JOIN json_each($2) n ON n.value=b.value) \
          OR EXISTS(SELECT 1 FROM json_each(h.after_node_ids) a JOIN json_each($2) n ON n.value=a.value))",
    )
    .bind(space_id)
    .bind(sqlx::types::Json(node_ids))
    .execute(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    Ok(())
}
