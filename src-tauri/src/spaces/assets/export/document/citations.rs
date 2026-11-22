use crate::spaces::db;

/// Reports whether `node_id` was produced as a completed grounded document, so export strips
/// Knovum's internal `[Cn]` workspace citation anchors instead of leaking them into standalone files.
pub(super) async fn is_grounded(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    node_id: &str,
) -> Result<bool, String> {
    let exists: Option<i64> = sqlx::query_scalar(
        "SELECT 1 FROM spaces_knowledge_derivations d JOIN spaces_grounded_documents g ON g.derivation_id=d.id WHERE d.output_node_id=$1 AND d.status='completed' LIMIT 1",
    )
    .bind(node_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    Ok(exists.is_some())
}
