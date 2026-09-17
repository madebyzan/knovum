use crate::spaces::db;

pub async fn validate(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    node_id: &str,
    expected: &str,
) -> Result<(), String> {
    let revision: Option<String> = sqlx::query_scalar(
        "SELECT COALESCE(json_extract(data,'$.knowledgeRevision'),json_extract(data,'$.sourceRevision')) FROM spaces_nodes WHERE id=$1 AND deleted_at IS NULL",
    )
    .bind(node_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(db::sql_error)?
    .flatten();
    if revision.as_deref() == Some(expected) {
        Ok(())
    } else {
        Err("Selected knowledge changed during derivation".into())
    }
}
