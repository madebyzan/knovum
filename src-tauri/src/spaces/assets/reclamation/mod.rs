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
    if candidates.is_empty() {
        return Ok(Vec::new());
    }
    let referenced = reference_graph::all(pool).await?;
    Ok(candidates
        .into_iter()
        .filter(|path| !referenced.contains(path))
        .collect())
}

#[cfg(test)]
mod tests;
