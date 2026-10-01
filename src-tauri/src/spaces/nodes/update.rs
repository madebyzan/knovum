use crate::app_error::AppError;
use serde_json::Value;
use sqlx::Row;
use tauri::{AppHandle, Runtime};

use super::{
    contracts::{NodeContentChange, UpdateNodesRequest, UpdateNodesResponse},
    derived::{changed_top_level_fields, field_changed, generated_chunks, media_chunks},
    space_timestamp::touch_updated_at,
};
use crate::spaces::{
    assets::enrichment::jobs,
    db,
    knowledge::{chunk_store, contracts, events},
};

pub(in crate::spaces) async fn update_many<R: Runtime>(
    app: &AppHandle<R>,
    mut request: UpdateNodesRequest,
) -> Result<UpdateNodesResponse, AppError> {
    if request.changes.is_empty() {
        return Ok(UpdateNodesResponse {
            nodes: Vec::new(),
            edit_snapshot: None,
        });
    }
    let pool = db::pool(app).await?;
    let mut tx = db::begin_write(pool).await?;
    for change in &mut request.changes {
        let kind = change.data["kind"]
            .as_str()
            .ok_or("Node kind is required")?
            .to_owned();
        contracts::normalize_node(&mut tx, &request.space_id, &kind, &mut change.data).await?;
    }
    let now = crate::clock::unix_time_millis()?;
    let edit_snapshot = update_many_in_transaction(
        &mut tx,
        &request.space_id,
        &request.changes,
        "human",
        now,
        jobs::intent::selection(app)?.as_ref(),
    )
    .await?;
    touch_updated_at(&mut tx, &request.space_id, now).await?;
    tx.commit().await.map_err(db::sql_error)?;
    Ok(UpdateNodesResponse {
        nodes: request
            .changes
            .into_iter()
            .map(|change| super::contracts::CommittedNodeContent {
                id: change.id,
                data: change.data,
            })
            .collect(),
        edit_snapshot,
    })
}

pub(in crate::spaces) async fn update_many_in_transaction(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    space_id: &str,
    changes: &[NodeContentChange],
    actor_kind: &str,
    now: i64,
    enrichment_target: Option<&crate::ai::tasks::TaskTargetSelection>,
) -> Result<Option<crate::spaces::structure::SpaceEditSnapshotResponse>, AppError> {
    for change in changes {
        apply_node_change(tx, space_id, change, actor_kind, now, enrichment_target).await?;
    }
    super::content_size::sync_changed_in_transaction(tx, space_id, changes).await
}

async fn apply_node_change(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    space_id: &str,
    change: &NodeContentChange,
    actor_kind: &str,
    now: i64,
    enrichment_target: Option<&crate::ai::tasks::TaskTargetSelection>,
) -> Result<(), String> {
    let row = sqlx::query("SELECT kind, data FROM spaces_nodes WHERE space_id = $1 AND id = $2 AND deleted_at IS NULL")
        .bind(space_id)
        .bind(&change.id)
        .fetch_optional(&mut **tx)
        .await
        .map_err(db::sql_error)?
        .ok_or_else(|| format!("Space node {} was not found", change.id))?;
    let kind: String = row.try_get("kind").map_err(db::sql_error)?;
    let previous_data: String = row.try_get("data").map_err(db::sql_error)?;
    let before: Value = serde_json::from_str(&previous_data).map_err(db::json_error)?;
    if before != change.expected_data {
        return Err(format!(
            "Space node {} changed before this edit was saved",
            change.id
        ));
    }
    let object_type_id = contracts::validate_node(tx, space_id, &kind, &change.data).await?;
    let revision_changed = field_changed(&before, &change.data, "knowledgeRevision");
    let source_revision_changed = field_changed(&before, &change.data, "sourceRevision");
    if revision_changed {
        chunk_store::detach_canonical(tx, space_id, &change.id).await?;
    }
    let data = serde_json::to_string(&change.data).map_err(db::json_error)?;
    sqlx::query("UPDATE spaces_nodes SET object_type_id = $1, data = $2, updated_at = $3 WHERE space_id = $4 AND id = $5")
        .bind(object_type_id)
        .bind(data)
        .bind(now)
        .bind(space_id)
        .bind(&change.id)
        .execute(&mut **tx)
        .await
        .map_err(db::sql_error)?;
    if let Some(source) = change.source_chunks.as_deref() {
        chunk_store::replace(tx, space_id, &change.id, "source", source, now).await?;
    } else if source_revision_changed {
        chunk_store::clear(tx, space_id, &change.id, "source").await?;
    }
    if revision_changed {
        let derived = generated_chunks(&change.data)?;
        chunk_store::replace(tx, space_id, &change.id, "derived", &derived, now).await?;
    }
    if field_changed(&before, &change.data, "mediaKnowledge") || source_revision_changed {
        let media = media_chunks(&change.data)?;
        let current_media = change
            .data
            .get("mediaKnowledge")
            .and_then(|value| value.get("sourceRevision"))
            .and_then(Value::as_str)
            == change.data.get("sourceRevision").and_then(Value::as_str);
        if current_media {
            chunk_store::replace(tx, space_id, &change.id, "media", &media, now).await?;
        } else {
            chunk_store::clear(tx, space_id, &change.id, "media").await?;
        }
    }
    events::append(
        tx,
        events::KnowledgeEvent::new(
            space_id,
            events::EventEntity::new("node", &change.id),
            "updated",
            events::EventActor::new(actor_kind, None),
            &serde_json::json!({
                "changedFields": changed_top_level_fields(&before, &change.data),
            }),
            now,
        ),
    )
    .await?;
    events::append_object_status_transition(
        tx,
        space_id,
        &change.id,
        &before,
        &change.data,
        events::EventActor::new(actor_kind, None),
        now,
    )
    .await?;
    let enrichment_changed = source_revision_changed
        || jobs::revision::visual_revision(&before)?
            != jobs::revision::visual_revision(&change.data)?
        || [
            "processingState",
            "mediaAnalysis",
            "mediaKnowledge",
            "visualKnowledge",
            "documentCoverage",
        ]
        .iter()
        .any(|field| field_changed(&before, &change.data, field));
    if enrichment_changed {
        jobs::intent::synchronize_for_node(
            tx,
            space_id,
            &change.id,
            &change.data,
            now,
            enrichment_target,
        )
        .await?;
    }
    Ok(())
}
