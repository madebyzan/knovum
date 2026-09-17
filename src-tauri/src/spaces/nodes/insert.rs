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
};

pub async fn insert_many<R: Runtime>(
    app: &AppHandle<R>,
    request: InsertNodesRequest,
) -> Result<(), String> {
    if request.nodes.is_empty() {
        return Ok(());
    }
    let now = crate::clock::unix_time_millis()?;
    let pool = db::pool(app).await?;
    let mut tx = db::begin_write(pool).await?;
    insert_many_in_transaction(&mut tx, &request.space_id, &request.nodes, now).await?;
    touch_updated_at(&mut tx, &request.space_id, now).await?;
    tx.commit().await.map_err(db::sql_error)
}

pub(in crate::spaces) async fn insert_many_in_transaction(
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
    if !node.initial_x.is_finite() || !node.initial_y.is_finite() {
        return Err("Space node initial placement must be finite".into());
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
