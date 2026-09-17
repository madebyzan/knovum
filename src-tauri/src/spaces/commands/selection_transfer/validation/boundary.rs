use std::collections::HashSet;

use sqlx::Row;

use crate::spaces::db;

pub(super) async fn validate_relations(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    space_id: &str,
    selected: &HashSet<&str>,
) -> Result<(), String> {
    let rows = sqlx::query("SELECT source_id,target_id FROM spaces_relations WHERE space_id=$1")
        .bind(space_id)
        .fetch_all(&mut **tx)
        .await
        .map_err(db::sql_error)?;
    for row in rows {
        let source: String = row.try_get("source_id").map_err(db::sql_error)?;
        let target: String = row.try_get("target_id").map_err(db::sql_error)?;
        if selected.contains(source.as_str()) != selected.contains(target.as_str()) {
            return Err(
                "Relations cannot be split across Spaces; move both endpoints together".into(),
            );
        }
    }
    Ok(())
}

pub(super) async fn validate_provenance(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    space_id: &str,
    selected: &HashSet<&str>,
) -> Result<(), String> {
    let rows = sqlx::query(
        "SELECT p.source_node_id,p.target_kind,p.target_id,r.source_id,r.target_id \
         FROM spaces_provenance p LEFT JOIN spaces_relations r \
         ON p.target_kind='relation' AND r.id=p.target_id WHERE p.space_id=$1",
    )
    .bind(space_id)
    .fetch_all(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    for row in rows {
        validate_provenance_row(row, selected)?;
    }
    Ok(())
}

fn validate_provenance_row(
    row: sqlx::sqlite::SqliteRow,
    selected: &HashSet<&str>,
) -> Result<(), String> {
    let source: String = row.try_get("source_node_id").map_err(db::sql_error)?;
    let target_kind: String = row.try_get("target_kind").map_err(db::sql_error)?;
    let source_moves = selected.contains(source.as_str());
    let target_moves = if target_kind == "node" {
        let target: String = row.try_get("target_id").map_err(db::sql_error)?;
        selected.contains(target.as_str())
    } else {
        let relation_source: Option<String> = row.try_get("source_id").map_err(db::sql_error)?;
        let relation_target: Option<String> = row.try_get("target_id").map_err(db::sql_error)?;
        relation_source
            .as_deref()
            .is_some_and(|id| selected.contains(id))
            && relation_target
                .as_deref()
                .is_some_and(|id| selected.contains(id))
    };
    if source_moves != target_moves {
        return Err("Provenance source and target must move together between Spaces".into());
    }
    Ok(())
}
