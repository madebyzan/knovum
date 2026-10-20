use sqlx::{Row, Sqlite, Transaction};

use crate::spaces::db;

/// Discarding content for good also discards the records that could bring it back.
///
/// A record names a node either through its presence or through the content row it would restore.
/// A coordinated record cannot be replayed on its own, so dropping one drops its partner with it.
/// The two statements are deliberate: a `DELETE` whose own rows feed its selection cannot be
/// trusted to see them all.
pub(crate) async fn invalidate_for_nodes(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    node_ids: &[String],
) -> Result<(), String> {
    let rows = sqlx::query(
        "SELECT h.id,h.history_group_id FROM spaces_edit_history h WHERE h.space_id=$1 AND ( \
         EXISTS(SELECT 1 FROM json_each(h.patch,'$.nodes') n \
           WHERE json_extract(n.value,'$.id') IN (SELECT value FROM json_each($2))) \
         OR EXISTS(SELECT 1 FROM json_each(h.patch,'$.structure') s \
           WHERE json_extract(s.value,'$.before.node.nodeId') IN (SELECT value FROM json_each($2)) \
              OR json_extract(s.value,'$.after.node.nodeId') IN (SELECT value FROM json_each($2))))",
    )
    .bind(space_id)
    .bind(sqlx::types::Json(node_ids))
    .fetch_all(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    let mut ids = Vec::new();
    let mut groups = Vec::new();
    for row in rows {
        ids.push(row.try_get::<i64, _>("id").map_err(db::sql_error)?);
        if let Some(group) = row
            .try_get::<Option<String>, _>("history_group_id")
            .map_err(db::sql_error)?
        {
            groups.push(group);
        }
    }
    if ids.is_empty() {
        return Ok(());
    }
    sqlx::query(
        "DELETE FROM spaces_edit_history \
         WHERE id IN (SELECT value FROM json_each($1)) \
            OR history_group_id IN (SELECT value FROM json_each($2))",
    )
    .bind(sqlx::types::Json(&ids))
    .bind(sqlx::types::Json(&groups))
    .execute(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    Ok(())
}
