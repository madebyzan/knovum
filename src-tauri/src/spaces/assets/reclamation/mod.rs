use std::collections::BTreeSet;

use sqlx::SqlitePool;

mod reference_graph;

pub(crate) use reference_graph::{
    for_nodes as referenced_paths_for_nodes, for_space as referenced_paths_for_space,
};

pub(crate) async fn unreferenced_paths(
    pool: &SqlitePool,
    candidates: BTreeSet<String>,
) -> Result<Vec<String>, String> {
    let mut tx = pool.begin().await.map_err(crate::spaces::db::sql_error)?;
    let paths = unreferenced_paths_in_transaction(&mut tx, candidates).await?;
    tx.commit().await.map_err(crate::spaces::db::sql_error)?;
    Ok(paths)
}

pub(crate) async fn unreferenced_paths_in_transaction(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    candidates: BTreeSet<String>,
) -> Result<Vec<String>, String> {
    if candidates.is_empty() {
        return Ok(Vec::new());
    }
    let referenced = reference_graph::all(tx).await?;
    Ok(candidates
        .into_iter()
        .filter(|path| !referenced.contains(path))
        .collect())
}

#[cfg(test)]
mod tests;
