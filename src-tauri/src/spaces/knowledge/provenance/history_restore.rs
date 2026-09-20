use sqlx::Row;
use crate::spaces::{db, knowledge::events};
use super::ProvenanceInput;
/** Restore the retained provenance identity only after validating its current endpoints and locator. */
pub(in crate::spaces) async fn restore(tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>, space_id: &str, id: &str, now: i64) -> Result<(), String> {
    let row = sqlx::query("SELECT target_kind,target_id,source_node_id,source_revision,locator,quote FROM spaces_provenance WHERE id=$1 AND space_id=$2 AND status='invalidated'")
        .bind(id).bind(space_id).fetch_optional(&mut **tx).await.map_err(db::sql_error)?.ok_or("Invalidated provenance is unavailable")?;
    let input = ProvenanceInput { id: id.into(), space_id: space_id.into(),
        target_kind: row.try_get("target_kind").map_err(db::sql_error)?, target_id: row.try_get("target_id").map_err(db::sql_error)?,
        source_node_id: row.try_get("source_node_id").map_err(db::sql_error)?, source_revision: row.try_get("source_revision").map_err(db::sql_error)?,
        locator: serde_json::from_str(&row.try_get::<String,_>("locator").map_err(db::sql_error)?).map_err(db::json_error)?, quote: row.try_get("quote").map_err(db::sql_error)?,
    };
    super::validate_input(&input)?; super::ensure_target(tx, &input).await?;
    let competing: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM spaces_provenance WHERE space_id=$1 AND target_kind=$2 AND target_id=$3 AND source_node_id=$4 AND status='active' AND id<>$5)")
        .bind(space_id).bind(&input.target_kind).bind(&input.target_id).bind(&input.source_node_id).bind(id).fetch_one(&mut **tx).await.map_err(db::sql_error)?;
    if competing { return Err("A newer provenance record now owns this relationship; the old edit cannot replace it".into()); }
    sqlx::query("UPDATE spaces_provenance SET status='active',updated_at=$1 WHERE id=$2 AND space_id=$3 AND status='invalidated'")
        .bind(now).bind(id).bind(space_id).execute(&mut **tx).await.map_err(db::sql_error)?;
    events::append(tx, events::EventRecord::new(space_id, events::EventEntity::new("provenance",id), "restored", events::EventActor::new("human",None), &serde_json::json!({"sourceRevision":input.source_revision}), now)).await
}
