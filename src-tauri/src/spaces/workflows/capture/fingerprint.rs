use std::collections::HashSet;

use sha2::{Digest, Sha256};
use sqlx::{QueryBuilder, Row, Sqlite, Transaction};

use super::contracts::{BoundaryInput, CapturedStep};
use crate::spaces::db;

pub(super) async fn calculate(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    selected: &HashSet<String>,
    visited: &HashSet<String>,
    steps: &[CapturedStep],
    inputs: &[BoundaryInput],
) -> Result<String, String> {
    let mut selected_ids = selected.iter().collect::<Vec<_>>();
    selected_ids.sort();
    let mut visited_ids = visited.iter().collect::<Vec<_>>();
    visited_ids.sort();

    let mut hasher = Sha256::new();
    let graph = serde_json::to_vec(&(&selected_ids, steps, inputs)).map_err(db::json_error)?;
    hasher.update(graph);

    let mut query = QueryBuilder::<Sqlite>::new(
        "SELECT id,kind,data,updated_at FROM spaces_nodes WHERE space_id=",
    );
    query.push_bind(space_id).push(" AND id IN (");
    let mut ids = query.separated(",");
    for id in &visited_ids {
        ids.push_bind(id.as_str());
    }
    ids.push_unseparated(") ORDER BY id");
    let rows = query
        .build()
        .fetch_all(&mut **tx)
        .await
        .map_err(db::sql_error)?;
    if rows.len() != visited.len() {
        return Err("workflow-capture-stale: a source resource changed during preview".into());
    }
    for row in rows {
        let snapshot = (
            row.try_get::<String, _>("id").map_err(db::sql_error)?,
            row.try_get::<String, _>("kind").map_err(db::sql_error)?,
            row.try_get::<String, _>("data").map_err(db::sql_error)?,
            row.try_get::<i64, _>("updated_at").map_err(db::sql_error)?,
        );
        hasher.update(serde_json::to_vec(&snapshot).map_err(db::json_error)?);
    }
    Ok(hex::encode(hasher.finalize()))
}
