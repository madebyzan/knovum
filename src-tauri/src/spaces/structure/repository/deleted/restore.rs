use super::RestoredStructure;

mod support;
use crate::spaces::db;
use sqlx::{Sqlite, Transaction};
pub(crate) use support::{mark_restored, remove_tombstones};
use support::reject_restore_conflicts;

pub(crate) async fn prepare_restore_batch(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    batch_id: &str,
) -> Result<RestoredStructure, String> {
    let exists:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM spaces_deleted_structure_batches WHERE id=$1 AND space_id=$2 AND restored_at IS NULL)").bind(batch_id).bind(space_id).fetch_one(&mut **tx).await.map_err(db::sql_error)?;
    if !exists {
        return Err("Deleted structure batch not found".into());
    }
    let structure_data: Vec<String> = sqlx::query_scalar(
        "SELECT data FROM spaces_deleted_structure_nodes WHERE batch_id=$1 ORDER BY structure_node_id",
    )
    .bind(batch_id)
    .fetch_all(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    let layout_data: Vec<String> = sqlx::query_scalar(
        "SELECT data FROM spaces_deleted_structure_layouts WHERE batch_id=$1 ORDER BY structure_node_id",
    )
    .bind(batch_id)
    .fetch_all(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    let structure = structure_data
        .into_iter()
        .map(|data| serde_json::from_str(&data).map_err(db::json_error))
        .collect::<Result<Vec<_>, _>>()?;
    let layouts = layout_data
        .into_iter()
        .map(|data| serde_json::from_str(&data).map_err(db::json_error))
        .collect::<Result<Vec<_>, _>>()?;
    reject_restore_conflicts(tx, space_id, &structure).await?;
    remove_tombstones(tx, space_id, &structure).await?;
    let batch_ids = vec![batch_id.to_owned()];
    let (relations, relation_styles) = support::load_relations(tx, &batch_ids).await?;
    Ok(RestoredStructure {
        batch_ids,
        structure,
        layouts,
        relation_styles,
        relations,
    })
}
