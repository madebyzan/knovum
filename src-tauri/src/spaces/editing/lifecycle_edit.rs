use crate::spaces::{db, knowledge::invalidation, nodes};
pub(super) async fn delete(tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>, space_id: &str, content_ids: &[String], relation_ids: &[String], now: i64) -> Result<(), String> {
    let mut relations = std::collections::HashSet::new();
    for id in relation_ids {
        if !relations.insert(id) { return Err("Duplicate relation deletion identity".into()); }
        let belongs: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM spaces_relations WHERE id=$1 AND space_id=$2 AND status<>'invalidated')").bind(id).bind(space_id).fetch_one(&mut **tx).await.map_err(db::sql_error)?;
        if !belongs { return Err("The selected relation is no longer active in this Space".into()); }
    }
    let removed = nodes::trash_in_transaction(tx, space_id, content_ids, now).await?;
    if removed.len() != content_ids.len() { return Err("Content deletion selection changed".into()); }
    for id in relation_ids { invalidation::invalidate_relation(tx, id, "human", None, now).await?; }
    Ok(())
}
