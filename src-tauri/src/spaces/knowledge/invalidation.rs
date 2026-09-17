use sqlx::{Row, Sqlite, Transaction};

use super::events;
use crate::spaces::db;

pub async fn invalidate_relation(
    tx: &mut Transaction<'_, Sqlite>,
    relation_id: &str,
    actor_kind: &str,
    actor_id: Option<&str>,
    now: i64,
) -> Result<bool, String> {
    let row = sqlx::query(
        "SELECT space_id,status FROM spaces_relations WHERE id=$1 AND status <> 'invalidated'",
    )
    .bind(relation_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    let Some(row) = row else {
        return Ok(false);
    };
    let space_id: String = row.try_get("space_id").map_err(db::sql_error)?;
    let prior_status: String = row.try_get("status").map_err(db::sql_error)?;
    sqlx::query("UPDATE spaces_relations SET status_before_invalidation=$1,status='invalidated',updated_at=$2 WHERE id=$3")
        .bind(&prior_status)
        .bind(now)
        .bind(relation_id)
        .execute(&mut **tx)
        .await
        .map_err(db::sql_error)?;
    events::append(
        tx,
        events::EventRecord::new(
            &space_id,
            events::EventEntity::new("relation", relation_id),
            "invalidated",
            events::EventActor::new(actor_kind, actor_id),
            &serde_json::json!({}),
            now,
        ),
    )
    .await?;
    invalidate_provenance_for_relation(tx, &space_id, relation_id, actor_kind, actor_id, now)
        .await?;
    Ok(true)
}

pub async fn restore_relation(
    tx: &mut Transaction<'_, Sqlite>,
    relation_id: &str,
    actor_kind: &str,
    actor_id: Option<&str>,
    now: i64,
) -> Result<String, String> {
    let row = sqlx::query(
        "SELECT space_id,source_id,target_id,status_before_invalidation FROM spaces_relations WHERE id=$1 AND status='invalidated'",
    )
    .bind(relation_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(db::sql_error)?
    .ok_or_else(|| "Invalidated Space Relation no longer exists".to_string())?;
    let space_id: String = row.try_get("space_id").map_err(db::sql_error)?;
    let source_id: String = row.try_get("source_id").map_err(db::sql_error)?;
    let target_id: String = row.try_get("target_id").map_err(db::sql_error)?;
    let restore_status: String = row
        .try_get::<Option<String>, _>("status_before_invalidation")
        .map_err(db::sql_error)?
        .ok_or_else(|| "Invalidated Space Relation is missing its prior status".to_string())?;
    super::contracts::validate_relation_status(&restore_status)?;
    if restore_status == "invalidated" {
        return Err("Invalid prior Space Relation status".to_string());
    }
    super::contracts::validate_relation_endpoints(tx, &space_id, &source_id, &target_id).await?;
    sqlx::query("UPDATE spaces_relations SET status=$1,status_before_invalidation=NULL,updated_at=$2 WHERE id=$3")
        .bind(&restore_status).bind(now).bind(relation_id)
        .execute(&mut **tx).await.map_err(db::sql_error)?;
    events::append(
        tx,
        events::EventRecord::new(
            &space_id,
            events::EventEntity::new("relation", relation_id),
            "restored",
            events::EventActor::new(actor_kind, actor_id),
            &serde_json::json!({"status": restore_status}),
            now,
        ),
    )
    .await?;
    Ok(restore_status)
}

pub async fn invalidate_provenance_for_relation(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    relation_id: &str,
    actor_kind: &str,
    actor_id: Option<&str>,
    now: i64,
) -> Result<(), String> {
    let rows = sqlx::query(
        "SELECT id FROM spaces_provenance WHERE space_id=$1 AND target_kind='relation' AND target_id=$2 AND status='active'",
    )
    .bind(space_id)
    .bind(relation_id)
    .fetch_all(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    for row in rows {
        let id: String = row.try_get("id").map_err(db::sql_error)?;
        invalidate_provenance(tx, space_id, &id, actor_kind, actor_id, now).await?;
    }
    Ok(())
}

pub async fn invalidate_provenance_for_nodes(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    node_ids: &[String],
    actor_kind: &str,
    actor_id: Option<&str>,
    now: i64,
) -> Result<(), String> {
    if node_ids.is_empty() {
        return Ok(());
    }
    let rows = sqlx::query(
        "SELECT id,source_node_id,target_kind,target_id FROM spaces_provenance WHERE space_id=$1 AND status='active'",
    )
    .bind(space_id)
    .fetch_all(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    let selected: std::collections::HashSet<&str> = node_ids.iter().map(String::as_str).collect();
    for row in rows {
        let id: String = row.try_get("id").map_err(db::sql_error)?;
        let source_node_id: String = row.try_get("source_node_id").map_err(db::sql_error)?;
        let target_kind: String = row.try_get("target_kind").map_err(db::sql_error)?;
        let target_id: String = row.try_get("target_id").map_err(db::sql_error)?;
        let touches_node = selected.contains(source_node_id.as_str())
            || (target_kind == "node" && selected.contains(target_id.as_str()));
        if touches_node {
            invalidate_provenance(tx, space_id, &id, actor_kind, actor_id, now).await?;
        }
    }
    Ok(())
}

async fn invalidate_provenance(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    provenance_id: &str,
    actor_kind: &str,
    actor_id: Option<&str>,
    now: i64,
) -> Result<(), String> {
    sqlx::query("UPDATE spaces_provenance SET status='invalidated',updated_at=$1 WHERE id=$2 AND status='active'")
        .bind(now)
        .bind(provenance_id)
        .execute(&mut **tx)
        .await
        .map_err(db::sql_error)?;
    events::append(
        tx,
        events::EventRecord::new(
            space_id,
            events::EventEntity::new("provenance", provenance_id),
            "invalidated",
            events::EventActor::new(actor_kind, actor_id),
            &serde_json::json!({}),
            now,
        ),
    )
    .await
}
