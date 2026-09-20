use crate::spaces::{db, nodes, knowledge::{invalidation, provenance}};
use super::lifecycle_state::LifecycleChanges;
pub(super) async fn apply(tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>, space_id: &str, changes: &LifecycleChanges, now: i64) -> Result<(), String> {
    let restored: Vec<_> = changes.nodes.iter().filter(|change| change.after.as_ref().is_some_and(|node| node.active)).map(|change| change.id.clone()).collect();
    let removed: Vec<_> = changes.nodes.iter().filter(|change| change.after.as_ref().is_none_or(|node| !node.active)).map(|change| change.id.clone()).collect();
    let actual = nodes::restore_in_transaction(tx, space_id, &restored, now).await?;
    if actual.len() != restored.len() { return Err("The restored content selection changed".into()); }
    let actual = nodes::trash_in_transaction(tx, space_id, &removed, now).await?;
    if actual.len() != removed.len() { return Err("The deleted content selection changed".into()); }
    for change in &changes.relations {
        if change.after.as_ref().is_some_and(|relation| relation.status != "invalidated") {
            let restored = invalidation::restore_relation(tx, &change.id, "human", None, now).await?;
            if change.after.as_ref().is_none_or(|relation| relation.status != restored) { return Err("The relation's prior state no longer matches its edit receipt".into()); }
        } else { invalidation::invalidate_relation(tx, &change.id, "human", None, now).await?; }
    }
    for change in &changes.provenance {
        if change.after.as_ref().is_some_and(|record| record.status == "active") { provenance::restore_for_history(tx, space_id, &change.id, now).await?; }
        else {
            let active: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM spaces_provenance WHERE id=$1 AND space_id=$2 AND status='active')").bind(&change.id).bind(space_id).fetch_one(&mut **tx).await.map_err(db::sql_error)?;
            if active { invalidation::invalidate_provenance(tx, space_id, &change.id, "human", None, now).await?; }
        }
    }
    Ok(())
}
