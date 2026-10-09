use lancedb::Table;
use sqlx::{Sqlite, Transaction};

use super::PendingInvalidation;
use crate::spaces::{
    db,
    knowledge::retrieval::predicates::{build_delete_node_predicate, build_delete_space_predicate},
};

pub(super) async fn apply(
    table: Option<&Table>,
    space_id: &str,
    pending: &[PendingInvalidation],
) -> Result<(), String> {
    let Some(table) = table else {
        return Ok(());
    };
    let predicate = if pending.iter().any(|item| item.node_id.is_none()) {
        build_delete_space_predicate(space_id)
    } else {
        let nodes = pending
            .iter()
            .filter_map(|item| item.node_id.clone())
            .collect::<Vec<_>>();
        build_delete_node_predicate(space_id, &nodes)
    };
    table
        .delete(&predicate)
        .await
        .map_err(|error| format!("Failed to delete LanceDB knowledge index rows: {error}"))?;
    Ok(())
}

pub(super) async fn acknowledge(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    pending: &[PendingInvalidation],
) -> Result<(), String> {
    let whole_space = pending.iter().any(|item| item.node_id.is_none());
    let nodes = pending
        .iter()
        .filter_map(|item| item.node_id.as_deref())
        .collect::<Vec<_>>();
    sqlx::query("DELETE FROM spaces_chunk_embeddings WHERE chunk_id IN (SELECT id FROM spaces_knowledge_chunks WHERE space_id=$1 AND ($2 OR node_id IN (SELECT value FROM json_each($3))))")
        .bind(space_id).bind(whole_space).bind(sqlx::types::Json(&nodes))
        .execute(&mut **tx).await.map_err(db::sql_error)?;
    let ids = pending.iter().map(|item| item.id).collect::<Vec<_>>();
    sqlx::query("DELETE FROM spaces_retrieval_invalidations WHERE space_id=$1 AND ($2 OR id IN (SELECT value FROM json_each($3)))")
        .bind(space_id).bind(whole_space).bind(sqlx::types::Json(&ids))
        .execute(&mut **tx).await.map_err(db::sql_error)?;
    Ok(())
}
