use tauri::{AppHandle, Runtime};

use super::{
    contracts::{InsertNode, InsertNodesRequest},
    derived::{generated_chunks, media_chunks},
    space_timestamp::touch_updated_at,
};
use crate::spaces::{
    assets::enrichment::jobs,
    db,
    knowledge::{chunk_store, contracts, events},
    structure,
};

pub async fn insert_many<R: Runtime>(
    app: &AppHandle<R>,
    request: InsertNodesRequest,
) -> Result<i64, String> {
    if request.nodes.is_empty() {
        return Err("Node creation requires at least one node".into());
    }
    if request.mutation_id.is_empty() || request.mutation_id.encode_utf16().count() > 128 {
        return Err("Mutation ID must contain 1 to 128 characters".into());
    }
    if request.expected_edit_revision < 0 {
        return Err("Expected edit revision must be nonnegative".into());
    }
    let now = crate::clock::unix_time_millis()?;
    let pool = db::pool(app).await?;
    let mut tx = db::begin_write(pool).await?;
    let digest = structure::receipts::digest(&request)?;
    if let Some(edit_revision) =
        structure::receipts::find_typed(&mut tx, &request.mutation_id, &digest).await?
    {
        tx.commit().await.map_err(db::sql_error)?;
        return Ok(edit_revision);
    }
    let (mut snapshot, revision) =
        structure::repository::load_in_transaction(&mut tx, &request.space_id).await?;
    if revision != request.expected_edit_revision {
        return Err("Space edit revision conflict".into());
    }
    let before = structure::history::capture(&mut tx, &request.space_id, &snapshot).await?;
    let selection = structure::creation::append_content(
        &mut snapshot,
        &request.nodes,
        request.target_parent_id.as_deref(),
    )?;
    insert_rows_in_transaction(&mut tx, &request.space_id, &request.nodes, now).await?;
    let edit_revision =
        structure::repository::save_in_transaction(&mut tx, &request.space_id, revision, &snapshot)
            .await?;
    structure::portal_integrity::validate_structure_targets(&mut tx).await?;
    let after = structure::history::capture(&mut tx, &request.space_id, &snapshot).await?;
    structure::history::record(
        &mut tx,
        &request.space_id,
        &before,
        &after,
        None,
        None,
        edit_revision,
    )
    .await?;
    structure::receipts::insert_typed(
        &mut tx,
        &request.mutation_id,
        &request.space_id,
        &digest,
        edit_revision,
        &selection,
        &edit_revision,
    )
    .await?;
    if !request.nodes.is_empty() {
        touch_updated_at(&mut tx, &request.space_id, now).await?;
    }
    tx.commit().await.map_err(db::sql_error)?;
    Ok(edit_revision)
}

pub(in crate::spaces) async fn insert_rows_in_transaction(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    space_id: &str,
    nodes: &[InsertNode],
    now: i64,
) -> Result<(), String> {
    for node in nodes {
        insert_single_node(tx, space_id, node, now).await?;
    }
    Ok(())
}

async fn insert_single_node(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    space_id: &str,
    node: &InsertNode,
    now: i64,
) -> Result<(), String> {
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
        contracts::validate_node(tx, space_id, &node.id, &node.kind, &node.data).await?;
    let data = serde_json::to_string(&node.data).map_err(db::json_error)?;
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
    .bind(data)
    .bind(now)
    .execute(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    events::append(
        tx,
        events::EventRecord::new(
            space_id,
            events::EventEntity::new("node", &node.id),
            "created",
            events::EventActor::new("human", None),
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
            events::EventActor::new("human", None),
            now,
        )
        .await?;
    }
    chunk_store::replace(tx, space_id, &node.id, "source", &node.source_chunks, now).await?;
    let derived = generated_chunks(&node.data)?;
    chunk_store::replace(tx, space_id, &node.id, "derived", &derived, now).await?;
    let media = media_chunks(&node.data)?;
    chunk_store::replace(tx, space_id, &node.id, "media", &media, now).await?;
    jobs::intent::synchronize_for_node(tx, space_id, &node.id, &node.data, now).await
}
