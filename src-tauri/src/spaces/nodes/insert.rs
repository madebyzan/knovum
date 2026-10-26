use crate::foundation::app_error::AppError;
use tauri::{AppHandle, Runtime};

use super::{
    contracts::{CommittedNodeContent, CreatedNodes, InsertNode, InsertNodesRequest},
    derived::{generated_chunks, media_chunks},
    space_timestamp::touch_updated_at,
};
use crate::spaces::{
    assets::enrichment::jobs,
    db,
    knowledge::{chunk_store, events},
    structure,
};

pub async fn insert_many<R: Runtime>(
    app: &AppHandle<R>,
    request: InsertNodesRequest,
) -> Result<CreatedNodes, AppError> {
    let pool = db::pool(app).await?;
    let mut tx = db::begin_write(pool).await?;
    let created =
        insert_many_in_transaction(&mut tx, request, jobs::intent::selection(app)?.as_ref())
            .await?;
    tx.commit().await.map_err(db::sql_error)?;
    Ok(created)
}

pub(in crate::spaces) async fn insert_many_in_transaction(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    mut request: InsertNodesRequest,
    enrichment_target: Option<&crate::ai::tasks::TaskTargetSelection>,
) -> Result<CreatedNodes, AppError> {
    if request.nodes.is_empty() {
        return Err("Node creation requires at least one node".into());
    }
    if request.mutation_id.is_empty() || request.mutation_id.encode_utf16().count() > 128 {
        return Err("Mutation ID must contain 1 to 128 characters".into());
    }
    let now = crate::foundation::clock::unix_time_millis()?;
    let digest = structure::receipts::digest(&request)?;
    if let Some(created) =
        structure::receipts::find_typed(tx, &request.mutation_id, &digest).await?
    {
        return Ok(created);
    }
    let (mut snapshot, revision) =
        structure::repository::load_in_transaction(tx, &request.space_id).await?;
    for node in &mut request.nodes {
        crate::spaces::knowledge::validation::normalize_node(
            tx,
            &request.space_id,
            &node.kind,
            &mut node.data,
        )
        .await?;
    }
    let before = structure::history::capture(tx, &request.space_id, &snapshot).await?;
    let selection = structure::creation::append_content(
        &mut snapshot,
        &request.nodes,
        request.target_parent_id.as_deref(),
    )?;
    insert_rows_in_transaction(
        tx,
        &request.space_id,
        &request.nodes,
        now,
        enrichment_target,
    )
    .await?;
    let edit_revision =
        structure::repository::save_in_transaction(tx, &request.space_id, revision, &snapshot)
            .await?;
    structure::portal_integrity::validate_structure_targets(tx, &[&request.space_id]).await?;
    let after = structure::history::capture(tx, &request.space_id, &snapshot).await?;
    structure::history::record(tx, &request.space_id, &before, &after).await?;
    let created = CreatedNodes {
        edit_revision,
        nodes: request
            .nodes
            .into_iter()
            .map(|node| CommittedNodeContent {
                id: node.id,
                data: node.data,
            })
            .collect(),
    };
    structure::receipts::insert_typed(
        tx,
        &request.mutation_id,
        &request.space_id,
        &digest,
        edit_revision,
        &selection,
        &created,
    )
    .await?;
    touch_updated_at(tx, &request.space_id, now).await?;
    Ok(created)
}

pub(in crate::spaces) async fn insert_rows_in_transaction(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    space_id: &str,
    nodes: &[InsertNode],
    now: i64,
    enrichment_target: Option<&crate::ai::tasks::TaskTargetSelection>,
) -> Result<(), AppError> {
    insert_rows_with_actor(
        tx,
        space_id,
        nodes,
        events::EventActor::new("human", None),
        now,
        enrichment_target,
    )
    .await
}

pub(in crate::spaces) async fn insert_rows_with_actor(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    space_id: &str,
    nodes: &[InsertNode],
    actor: events::EventActor<'_>,
    now: i64,
    enrichment_target: Option<&crate::ai::tasks::TaskTargetSelection>,
) -> Result<(), AppError> {
    for node in nodes {
        insert_single_node(tx, space_id, node, actor, now, enrichment_target).await?;
    }
    Ok(())
}

async fn insert_single_node(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    space_id: &str,
    node: &InsertNode,
    actor: events::EventActor<'_>,
    now: i64,
    enrichment_target: Option<&crate::ai::tasks::TaskTargetSelection>,
) -> Result<(), AppError> {
    if node.kind == "section" {
        return Err("Section is a structure node and cannot be inserted as content".into());
    }
    let fields = node
        .data
        .as_object()
        .ok_or("Space node data must be an object")?;
    if fields.contains_key("sectionId") || fields.contains_key("locked") {
        return Err("Content payload cannot contain structure fields sectionId or locked".into());
    }
    let object_type_id =
        crate::spaces::knowledge::validation::validate_node(tx, space_id, &node.kind, &node.data)
            .await?;
    sqlx::query(
        r#"INSERT INTO spaces_nodes (
             id, space_id, kind, object_type_id, initial_x, initial_y, data, created_at, updated_at
           ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$8)"#,
    )
    .bind(&node.id)
    .bind(space_id)
    .bind(&node.kind)
    .bind(object_type_id)
    .bind(node.initial_x)
    .bind(node.initial_y)
    .bind(sqlx::types::Json(&node.data))
    .bind(now)
    .execute(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    events::append(
        tx,
        events::KnowledgeEvent::new(
            space_id,
            events::EventEntity::new("node", &node.id),
            "created",
            actor,
            &serde_json::json!({"kind": node.kind}),
            now,
        ),
    )
    .await?;
    if node.kind == "object" {
        events::append_object_status_transition(
            tx,
            space_id,
            &node.id,
            &serde_json::json!({}),
            &node.data,
            actor,
            now,
        )
        .await?;
    }
    chunk_store::replace(tx, space_id, &node.id, "source", &node.source_chunks, now).await?;
    let derived = generated_chunks(&node.data)?;
    chunk_store::replace(tx, space_id, &node.id, "derived", &derived, now).await?;
    let media = media_chunks(&node.data)?;
    chunk_store::replace(tx, space_id, &node.id, "media", &media, now).await?;
    jobs::intent::synchronize_for_node(tx, space_id, &node.id, &node.data, now, enrichment_target)
        .await?;
    Ok(())
}
