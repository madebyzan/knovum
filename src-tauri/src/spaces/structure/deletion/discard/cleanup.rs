use crate::spaces::db;

pub(super) async fn reject_object_dependents(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    space_id: &str,
    node_id: &str,
) -> Result<(), String> {
    let rows = sqlx::query(
        "SELECT n.id,n.data,t.fields FROM spaces_nodes n \
         LEFT JOIN spaces_object_types t ON t.id=n.object_type_id \
         WHERE n.space_id=$1 AND n.kind='object' AND n.deleted_at IS NULL AND n.id<>$2",
    )
    .bind(space_id)
    .bind(node_id)
    .fetch_all(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    for row in rows {
        let id: String = row.try_get("id").map_err(db::sql_error)?;
        let raw: String = row.try_get("data").map_err(db::sql_error)?;
        let fields: Option<String> = row.try_get("fields").map_err(db::sql_error)?;
        let data = serde_json::from_str(&raw).map_err(db::json_error)?;
        if crate::spaces::knowledge::object_references::collect_node_ids(fields.as_deref(), &data)?
            .iter()
            .any(|target| target == node_id)
        {
            return Err(format!(
                "Retained object {id} must be edited before discarding its referenced content"
            ));
        }
    }
    Ok(())
}
use sqlx::Row;
use std::collections::HashSet;

pub(super) async fn relation_ids(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    space_id: &str,
    node_id: &str,
) -> Result<HashSet<String>, String> {
    let ids = sqlx::query_scalar(
        "SELECT id FROM spaces_relations WHERE space_id=$1 AND (source_id=$2 OR target_id=$2)",
    )
    .bind(space_id)
    .bind(node_id)
    .fetch_all(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    Ok(ids.into_iter().collect())
}

pub(super) async fn provenance_ids(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    space_id: &str,
    node_id: &str,
    relations: &HashSet<String>,
) -> Result<HashSet<String>, String> {
    let rows = sqlx::query(
        "SELECT id,source_node_id,target_kind,target_id FROM spaces_provenance WHERE space_id=$1",
    )
    .bind(space_id)
    .fetch_all(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    let mut ids = HashSet::new();
    for row in rows {
        let source: String = row.try_get("source_node_id").map_err(db::sql_error)?;
        let kind: String = row.try_get("target_kind").map_err(db::sql_error)?;
        let target: String = row.try_get("target_id").map_err(db::sql_error)?;
        if source == node_id
            || (kind == "node" && target == node_id)
            || (kind == "relation" && relations.contains(&target))
        {
            ids.insert(row.try_get("id").map_err(db::sql_error)?);
        }
    }
    Ok(ids)
}

pub(super) async fn delete_related(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    relations: &HashSet<String>,
    provenance: &HashSet<String>,
    node_id: &str,
) -> Result<(), String> {
    for (kind, ids) in [("relation", relations), ("provenance", provenance)] {
        for id in ids {
            sqlx::query(
                "DELETE FROM spaces_knowledge_events WHERE entity_kind=$1 AND entity_id=$2",
            )
            .bind(kind)
            .bind(id)
            .execute(&mut **tx)
            .await
            .map_err(db::sql_error)?;
        }
    }
    sqlx::query("DELETE FROM spaces_knowledge_events WHERE entity_kind='node' AND entity_id=$1")
        .bind(node_id)
        .execute(&mut **tx)
        .await
        .map_err(db::sql_error)?;
    for id in provenance {
        sqlx::query("DELETE FROM spaces_provenance WHERE id=$1")
            .bind(id)
            .execute(&mut **tx)
            .await
            .map_err(db::sql_error)?;
    }
    for id in relations {
        sqlx::query("DELETE FROM spaces_relations WHERE id=$1")
            .bind(id)
            .execute(&mut **tx)
            .await
            .map_err(db::sql_error)?;
    }
    Ok(())
}
