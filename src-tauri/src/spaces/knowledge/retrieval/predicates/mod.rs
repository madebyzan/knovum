use super::schema::RetrievalRecord;

/// Escapes single quotes in a SQL literal by doubling them.
pub(super) fn escape_sql_literal(value: &str) -> String {
    value.replace('\'', "''")
}

/// Builds a LanceDB filter predicate that removes every row belonging to the given nodes in a space.
pub(super) fn build_delete_node_predicate(space_id: &str, node_ids: &[String]) -> String {
    let ids = node_ids
        .iter()
        .map(|id| format!("'{}'", escape_sql_literal(id)))
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "space_id = '{}' AND node_id IN ({ids})",
        escape_sql_literal(space_id)
    )
}

/// Builds a LanceDB filter predicate that removes all rows belonging to a space.
pub(super) fn build_delete_space_predicate(space_id: &str) -> String {
    format!("space_id = '{}'", escape_sql_literal(space_id))
}

/// Builds a LanceDB filter predicate that removes stale rows sharing chunk IDs with the batch.
pub(super) fn build_delete_chunk_predicate(records: &[RetrievalRecord]) -> String {
    let chunk_ids = records
        .iter()
        .map(|record| format!("'{}'", escape_sql_literal(&record.chunk_id)))
        .collect::<Vec<_>>()
        .join(",");
    format!("chunk_id IN ({chunk_ids})")
}

#[cfg(test)]
mod tests;
