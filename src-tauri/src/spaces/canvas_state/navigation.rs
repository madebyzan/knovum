use crate::spaces::db;

pub(in crate::spaces) async fn validate_item_target(tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>, space_id: &str, id: &str) -> Result<(), String> {
    let (document, _) = super::load_in_transaction(tx, space_id).await?;
    if !document.structure.items.iter().any(|item| item.id == id) { return Err("Navigation target must be an active structural item in its Space".into()); }
    Ok(())
}
pub(in crate::spaces) async fn ensure_existing_space(tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>, space_id: &str) -> Result<(), String> {
    let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM spaces WHERE id=$1)").bind(space_id).fetch_one(&mut **tx).await.map_err(db::sql_error)?;
    if !exists { return Err("The target Space does not exist".into()); }
    Ok(())
}
