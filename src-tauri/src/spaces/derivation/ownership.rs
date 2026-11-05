//! A derivation records how its output was made, so it belongs to the Space its output is in.

use sqlx::{types::Json, Sqlite, Transaction};

use crate::spaces::db;

/// Brings the derivations of `node_ids` into the Space their outputs are now in, once those nodes
/// have moved. An input keeps its node only while both share the derivation's Space; its recorded
/// snapshot stays either way, so a derivation read across Spaces is still told by what it read.
pub(crate) async fn follow_moved_nodes(
    tx: &mut Transaction<'_, Sqlite>,
    node_ids: &[String],
    now: i64,
) -> Result<(), String> {
    sqlx::query(
        "UPDATE spaces_knowledge_derivations \
         SET space_id=(SELECT space_id FROM spaces_nodes WHERE id=output_node_id),updated_at=$1 \
         WHERE output_node_id IN (SELECT value FROM json_each($2)) \
           AND space_id<>(SELECT space_id FROM spaces_nodes WHERE id=output_node_id)",
    )
    .bind(now)
    .bind(Json(node_ids))
    .execute(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    sqlx::query(
        "UPDATE spaces_knowledge_derivation_inputs SET node_id=NULL \
         WHERE (node_id IN (SELECT value FROM json_each($1)) \
            OR derivation_id IN (SELECT id FROM spaces_knowledge_derivations \
                                 WHERE output_node_id IN (SELECT value FROM json_each($1)))) \
           AND NOT EXISTS (SELECT 1 FROM spaces_knowledge_derivations d \
                           JOIN spaces_nodes n ON n.space_id=d.space_id \
                           WHERE d.id=spaces_knowledge_derivation_inputs.derivation_id \
                             AND n.id=spaces_knowledge_derivation_inputs.node_id)",
    )
    .bind(Json(node_ids))
    .execute(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    Ok(())
}
