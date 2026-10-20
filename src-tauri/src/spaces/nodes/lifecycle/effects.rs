use std::collections::HashSet;

use sqlx::Row;

use crate::spaces::{db, knowledge::invalidation};

pub(super) async fn invalidate_relations_for_nodes(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    space_id: &str,
    node_ids: &[String],
    now: i64,
) -> Result<(), String> {
    // Two indexed lookups, one per endpoint. A single OR over both endpoints cannot use either
    // index, which made trashing one node read every relation in the Space.
    let ids: Vec<String> = sqlx::query_scalar(
        "SELECT id FROM spaces_relations \
         WHERE space_id=$1 AND status <> 'invalidated' \
           AND source_id IN (SELECT value FROM json_each($2)) \
         UNION \
         SELECT id FROM spaces_relations \
         WHERE space_id=$1 AND status <> 'invalidated' \
           AND target_id IN (SELECT value FROM json_each($2))",
    )
    .bind(space_id)
    .bind(sqlx::types::Json(node_ids))
    .fetch_all(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    for id in ids {
        invalidation::invalidate_relation(tx, &id, "human", None, now).await?;
    }
    Ok(())
}

pub(super) async fn cancel_enrichment_jobs_for_nodes(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    space_id: &str,
    node_ids: &[String],
    now: i64,
) -> Result<(), String> {
    sqlx::query(
        "UPDATE spaces_enrichment_jobs SET status='cancelled', updated_at=$1 \
         WHERE space_id=$2 AND node_id IN (SELECT value FROM json_each($3)) \
           AND status IN ('pending','running','awaiting_configuration')",
    )
    .bind(now)
    .bind(space_id)
    .bind(sqlx::types::Json(node_ids))
    .execute(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    Ok(())
}

pub(super) async fn remove_nodes_from_scene_focus(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    space_id: &str,
    node_ids: &[String],
    now: i64,
) -> Result<(), String> {
    let removed_node_ids: HashSet<&str> = node_ids.iter().map(String::as_str).collect();
    // Only the scenes that actually focus one of these nodes need rewriting, so the cost follows
    // the deletion rather than the number of scenes in the Space.
    let rows = sqlx::query(
        "SELECT id,focus_structure_node_ids FROM spaces_scenes \
         WHERE space_id=$1 AND EXISTS ( \
           SELECT 1 FROM json_each(focus_structure_node_ids) \
           WHERE value IN (SELECT value FROM json_each($2)))",
    )
    .bind(space_id)
    .bind(sqlx::types::Json(node_ids))
    .fetch_all(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    for row in rows {
        let id: String = row.try_get("id").map_err(db::sql_error)?;
        let raw: String = row
            .try_get("focus_structure_node_ids")
            .map_err(db::sql_error)?;
        let mut focus: Vec<String> = serde_json::from_str(&raw).map_err(db::json_error)?;
        let before = focus.len();
        focus.retain(|structure_node_id| !removed_node_ids.contains(structure_node_id.as_str()));
        if focus.len() == before {
            continue;
        }
        sqlx::query(
            "UPDATE spaces_scenes SET focus_structure_node_ids=$1,updated_at=$2 WHERE id=$3",
        )
        .bind(sqlx::types::Json(&focus))
        .bind(now)
        .bind(id)
        .execute(&mut **tx)
        .await
        .map_err(db::sql_error)?;
    }
    Ok(())
}

pub(super) async fn mark_nodes_deleted(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    space_id: &str,
    node_ids: &[String],
    now: i64,
) -> Result<(), String> {
    super::validation::reject_live_dependents(tx, space_id, node_ids).await?;
    crate::spaces::knowledge::grounding::references::detach_nodes(tx, space_id, node_ids).await?;
    invalidate_relations_for_nodes(tx, space_id, node_ids, now).await?;
    invalidation::invalidate_provenance_for_nodes(tx, space_id, node_ids, "human", None, now)
        .await?;
    remove_nodes_from_scene_focus(tx, space_id, node_ids, now).await?;
    cancel_enrichment_jobs_for_nodes(tx, space_id, node_ids, now).await?;
    crate::spaces::derivation::pending_output::stop_runs_owing(tx, space_id, node_ids, now).await?;
    for id in node_ids {
        sqlx::query(
            "UPDATE spaces_nodes SET deleted_at=$1,updated_at=$1 WHERE id=$2 AND space_id=$3",
        )
        .bind(now)
        .bind(id)
        .bind(space_id)
        .execute(&mut **tx)
        .await
        .map_err(db::sql_error)?;
        crate::spaces::knowledge::events::append(
            tx,
            crate::spaces::knowledge::events::KnowledgeEvent::new(
                space_id,
                crate::spaces::knowledge::events::EventEntity::new("node", id),
                "invalidated",
                crate::spaces::knowledge::events::EventActor::new("human", None),
                &serde_json::json!({}),
                now,
            ),
        )
        .await?;
    }
    Ok(())
}

pub(super) async fn restore_nodes(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    space_id: &str,
    node_ids: &[String],
    now: i64,
) -> Result<(), String> {
    for id in node_ids {
        sqlx::query(
            "UPDATE spaces_nodes SET deleted_at=NULL,updated_at=$1 WHERE id=$2 AND space_id=$3",
        )
        .bind(now)
        .bind(id)
        .bind(space_id)
        .execute(&mut **tx)
        .await
        .map_err(db::sql_error)?;
    }
    for id in node_ids {
        let row = sqlx::query(
            "SELECT kind,data FROM spaces_nodes WHERE id=$1 AND space_id=$2 AND deleted_at IS NULL",
        )
        .bind(id)
        .bind(space_id)
        .fetch_one(&mut **tx)
        .await
        .map_err(db::sql_error)?;
        let kind: String = row.try_get("kind").map_err(db::sql_error)?;
        let raw: String = row.try_get("data").map_err(db::sql_error)?;
        let data: serde_json::Value = serde_json::from_str(&raw).map_err(db::json_error)?;
        crate::spaces::knowledge::validation::validate_node(tx, space_id, &kind, &data).await?;
        crate::spaces::knowledge::events::append(
            tx,
            crate::spaces::knowledge::events::KnowledgeEvent::new(
                space_id,
                crate::spaces::knowledge::events::EventEntity::new("node", id),
                "restored",
                crate::spaces::knowledge::events::EventActor::new("human", None),
                &serde_json::json!({}),
                now,
            ),
        )
        .await?;
    }
    Ok(())
}
