use crate::spaces::{db, structure::StructureNode};
use sqlx::{Sqlite, Transaction};
use std::collections::HashSet;

pub(super) async fn invalidate(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    structure: &[StructureNode],
    nodes: &HashSet<String>,
) -> Result<(), String> {
    let structure_node_ids = structure.iter().map(StructureNode::id).collect::<Vec<_>>();
    let match_sql = "(EXISTS(SELECT 1 FROM json_each(h.before_snapshot,'$.structure') i JOIN json_each($2) d ON d.value=json_extract(i.value,'$.id')) OR EXISTS(SELECT 1 FROM json_each(h.after_snapshot,'$.structure') i JOIN json_each($2) d ON d.value=json_extract(i.value,'$.id')) OR EXISTS(SELECT 1 FROM json_each(h.before_node_ids) n JOIN json_each($3) d ON d.value=n.value) OR EXISTS(SELECT 1 FROM json_each(h.after_node_ids) n JOIN json_each($3) d ON d.value=n.value))";
    let mut groups_query = sqlx::QueryBuilder::<Sqlite>::new("SELECT DISTINCT history_group_id FROM spaces_edit_history h WHERE h.space_id=$1 AND h.history_group_id IS NOT NULL AND ");
    groups_query.push(match_sql);
    let groups: Vec<String> = groups_query
        .build_query_scalar()
        .bind(space_id)
        .bind(sqlx::types::Json(&structure_node_ids))
        .bind(sqlx::types::Json(nodes))
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
    let mut deletion = sqlx::QueryBuilder::<Sqlite>::new(
        "DELETE FROM spaces_edit_history AS h WHERE h.space_id=$1 AND ",
    );
    deletion.push(match_sql);
    deletion
        .build()
        .bind(space_id)
        .bind(sqlx::types::Json(&structure_node_ids))
        .bind(sqlx::types::Json(nodes))
        .execute(&mut **tx)
        .await
        .map_err(db::sql_error)?;
    Ok(())
}
