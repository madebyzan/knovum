pub(super) mod contracts;
mod operations;
mod validation;

use std::collections::HashSet;

use tauri::{AppHandle, Runtime};

use crate::spaces::{
    db,
    knowledge::{self, events},
};
use contracts::TransferSelectionRequest;

pub async fn transfer_selection<R: Runtime>(
    app: &AppHandle<R>,
    request: TransferSelectionRequest,
) -> Result<Vec<String>, String> {
    if request.source_space_id == request.target_space_id || request.ids.is_empty() {
        return Ok(Vec::new());
    }
    let now = crate::clock::unix_time_millis()?;
    let pool = db::pool(app).await?;
    let mut tx = db::begin_write(pool).await?;
    validation::ensure_target_space(&mut tx, &request.target_space_id).await?;
    let selected = validation::selected_nodes(&mut tx, &request).await?;
    if selected.is_empty() {
        return Ok(Vec::new());
    }
    let selected_set: HashSet<&str> = selected.iter().map(String::as_str).collect();

    validation::validate_dependencies(&mut tx, &request.source_space_id, &selected).await?;
    knowledge::grounding::references::detach_nodes(&mut tx, &request.source_space_id, &selected)
        .await?;
    operations::update_portals(&mut tx, &request, &selected_set, now).await?;
    let moved_relation_ids =
        operations::moved_relation_ids(&mut tx, &request.source_space_id, &selected_set).await?;
    let moved_provenance_ids =
        operations::moved_provenance_ids(&mut tx, &request.source_space_id, &selected_set).await?;

    // Move nodes first. Database triggers enforce same-Space endpoints on dependent rows,
    // so dependents must only change Space after their referenced nodes have moved.
    operations::remove_scene_focus(&mut tx, &request.source_space_id, &selected_set, now).await?;
    for id in &selected {
        sqlx::query(
            "UPDATE spaces_nodes SET space_id=$1,data=json_remove(data,'$.sectionId'),updated_at=$2 WHERE id=$3 AND space_id=$4",
        ).bind(&request.target_space_id).bind(now).bind(id).bind(&request.source_space_id)
            .execute(&mut *tx).await.map_err(db::sql_error)?;
    }

    let selected_json = serde_json::to_string(&selected).map_err(db::json_error)?;
    sqlx::query("UPDATE spaces_knowledge_derivation_inputs SET node_id=NULL WHERE node_id IN (SELECT value FROM json_each($1)) AND derivation_id IN (SELECT id FROM spaces_knowledge_derivations WHERE space_id=$2)")
        .bind(&selected_json).bind(&request.source_space_id)
        .execute(&mut *tx).await.map_err(db::sql_error)?;
    sqlx::query(
        "UPDATE spaces_relations SET space_id=$1,updated_at=$2 WHERE space_id=$3 \
         AND source_id IN (SELECT value FROM json_each($4)) \
         AND target_id IN (SELECT value FROM json_each($4))",
    )
    .bind(&request.target_space_id)
    .bind(now)
    .bind(&request.source_space_id)
    .bind(&selected_json)
    .execute(&mut *tx)
    .await
    .map_err(db::sql_error)?;
    operations::move_provenance(&mut tx, &request, &selected_set, now).await?;
    sqlx::query(
        "DELETE FROM spaces_chunk_embeddings WHERE chunk_id IN ( \
         SELECT id FROM spaces_knowledge_chunks WHERE node_id IN (SELECT value FROM json_each($1)) \
        )",
    )
    .bind(&selected_json)
    .execute(&mut *tx)
    .await
    .map_err(db::sql_error)?;
    append_moved_events(&mut tx, &request.target_space_id, "node", &selected, now).await?;
    append_moved_events(
        &mut tx,
        &request.target_space_id,
        "relation",
        &moved_relation_ids,
        now,
    )
    .await?;
    append_moved_events(
        &mut tx,
        &request.target_space_id,
        "provenance",
        &moved_provenance_ids,
        now,
    )
    .await?;
    for space_id in [&request.source_space_id, &request.target_space_id] {
        sqlx::query("UPDATE spaces SET updated_at=$1 WHERE id=$2")
            .bind(now)
            .bind(space_id)
            .execute(&mut *tx)
            .await
            .map_err(db::sql_error)?;
    }
    crate::spaces::knowledge::retrieval::invalidation_outbox::enqueue_nodes(
        &mut tx,
        &request.source_space_id,
        &selected,
        now,
    )
    .await?;
    tx.commit().await.map_err(db::sql_error)?;
    crate::spaces::knowledge::retrieval::invalidation_outbox::drain_and_report(app, pool).await;
    Ok(selected)
}

async fn append_moved_events(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    space_id: &str,
    entity_kind: &str,
    ids: &[String],
    now: i64,
) -> Result<(), String> {
    let empty_payload = serde_json::json!({});
    for id in ids {
        events::append(
            tx,
            events::EventRecord::new(
                space_id,
                events::EventEntity::new(entity_kind, id),
                "moved",
                events::EventActor::new("human", None),
                &empty_payload,
                now,
            ),
        )
        .await?;
    }
    Ok(())
}
