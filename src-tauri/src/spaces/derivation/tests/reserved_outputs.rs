//! What happens to a Run's output node when it is reserved, published, deleted or abandoned.

use serde_json::json;

use super::running_action_pool;
use crate::foundation::app_error::AppError;
use crate::spaces::{
    db,
    derivation::{
        pending_output::{self, PendingOutput},
        persist_output_node,
    },
    nodes, structure,
};

const RESERVED: &str = "run-output";

fn reservation() -> PendingOutput {
    PendingOutput {
        node_id: RESERVED.into(),
        kind: "image",
        data: json!({
            "kind":"image","title":"Upscaled","fileName":"upscaled.png",
            "assetPath":"/managed/upscaled.png","processingState":"processing"
        }),
        placement: pending_output::OutputPlacement::Derived(pending_output::PLACEHOLDER),
    }
}

async fn reserve(tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>) -> Result<(), AppError> {
    pending_output::reserve_in_transaction(
        tx,
        "space",
        vec![reservation()],
        &["source".to_owned()],
        "Image upscaled from its source",
        "run",
        None,
    )
    .await
}

async fn space_with_reserved_output() -> sqlx::SqlitePool {
    let pool = running_action_pool("run").await;
    let mut tx = db::begin_write(&pool).await.unwrap();
    structure::repository::initialize_empty_in_transaction(&mut tx, "space")
        .await
        .unwrap();
    nodes::tests::insert_source_in_transaction(
        &mut tx,
        "space",
        "source",
        "note",
        nodes::tests::source_note("Source"),
    )
    .await
    .unwrap();
    reserve(&mut tx).await.unwrap();
    tx.commit().await.unwrap();
    pool
}

async fn trash(pool: &sqlx::SqlitePool, node_id: &str) {
    let mut tx = db::begin_write(pool).await.unwrap();
    let trashed =
        crate::spaces::nodes::trash_in_transaction(&mut tx, "space", &[node_id.into()], 2)
            .await
            .unwrap();
    assert_eq!(trashed, [node_id.to_owned()]);
    sqlx::query("UPDATE spaces_structure_nodes SET deleted_at=2 WHERE node_id=$1")
        .bind(node_id)
        .execute(&mut *tx)
        .await
        .unwrap();
    tx.commit().await.unwrap();
}

fn published() -> serde_json::Value {
    json!({
        "kind":"image","title":"Upscaled","fileName":"upscaled.png",
        "assetPath":"/managed/upscaled.png","sourceRevision":"sha256:upscaled",
        "size":1024,"modifiedAt":3,"intrinsicWidth":2,"intrinsicHeight":2,
        "knowledgeRevision":"r1"
    })
}

#[tokio::test]
async fn deleting_a_reserved_output_calls_off_the_run_that_owes_it() {
    let pool = space_with_reserved_output().await;
    trash(&pool, RESERVED).await;
    let cancelled: Option<i64> =
        sqlx::query_scalar("SELECT cancel_requested_at FROM spaces_runs WHERE id='run'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(
        cancelled.is_some(),
        "work nobody is waiting for must be called off"
    );
}

#[tokio::test]
async fn publishing_into_a_deleted_output_fails_instead_of_reporting_success() {
    let pool = space_with_reserved_output().await;
    trash(&pool, RESERVED).await;
    let mut tx = db::begin_write(&pool).await.unwrap();
    let error = persist_output_node(&mut tx, RESERVED, &published(), 3, None)
        .await
        .unwrap_err();
    assert!(error.message.contains("deleted"), "{}", error.message);
}

#[tokio::test]
async fn an_output_deleted_before_its_run_ends_is_not_resurrected_by_a_retry() {
    let pool = space_with_reserved_output().await;
    trash(&pool, RESERVED).await;
    let mut tx = db::begin_write(&pool).await.unwrap();
    let error = reserve(&mut tx).await.unwrap_err();
    assert!(error.message.contains("deleted"), "{}", error.message);
}

#[tokio::test]
async fn a_restored_output_shows_the_work_is_over_rather_than_waiting_forever() {
    let pool = space_with_reserved_output().await;
    trash(&pool, RESERVED).await;
    sqlx::query("UPDATE spaces_runs SET state='cancelled',completed_at=4 WHERE id='run'")
        .execute(&pool)
        .await
        .unwrap();
    pending_output::settle_abandoned(&pool).await.unwrap();
    let mut tx = db::begin_write(&pool).await.unwrap();
    crate::spaces::nodes::restore_in_transaction(&mut tx, "space", &[RESERVED.into()], 5)
        .await
        .unwrap();
    tx.commit().await.unwrap();
    let state: Option<String> = sqlx::query_scalar(
        "SELECT json_extract(data,'$.processingState') FROM spaces_nodes WHERE id=$1",
    )
    .bind(RESERVED)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(state.as_deref(), Some("failed"));
}

#[tokio::test]
async fn a_published_output_stops_waiting_and_survives_its_run_ending() {
    let pool = space_with_reserved_output().await;
    let mut tx = db::begin_write(&pool).await.unwrap();
    persist_output_node(&mut tx, RESERVED, &published(), 3, None)
        .await
        .unwrap();
    tx.commit().await.unwrap();
    let waiting: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM spaces_reserved_outputs")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(waiting, 0);
    sqlx::query("UPDATE spaces_runs SET state='failed',completed_at=4 WHERE id='run'")
        .execute(&pool)
        .await
        .unwrap();
    pending_output::settle_abandoned(&pool).await.unwrap();
    let state: Option<String> = sqlx::query_scalar(
        "SELECT json_extract(data,'$.processingState') FROM spaces_nodes WHERE id=$1",
    )
    .bind(RESERVED)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(state, None, "a settled output is not failed afterwards");
}

#[tokio::test]
async fn a_framed_output_is_reserved_in_its_frame_and_claims_no_source_it_lacks() {
    let pool = running_action_pool("run").await;
    let mut tx = db::begin_write(&pool).await.unwrap();
    structure::repository::initialize_empty_in_transaction(&mut tx, "space")
        .await
        .unwrap();
    pending_output::reserve_in_transaction(
        &mut tx,
        "space",
        vec![PendingOutput {
            placement: pending_output::OutputPlacement::Framed(
                serde_json::from_value(json!({"x":120.0,"y":-40.0,"width":320.0,"height":180.0}))
                    .unwrap(),
            ),
            ..reservation()
        }],
        &[],
        "Image generated from its prompt",
        "run",
        None,
    )
    .await
    .unwrap();
    tx.commit().await.unwrap();
    let placed: (f64, f64, f64, f64) = sqlx::query_as(
        "SELECT json_extract(layout.data,'$.transform.x'),json_extract(layout.data,'$.transform.y'),\
         json_extract(layout.data,'$.size.width'),json_extract(layout.data,'$.size.height') \
         FROM spaces_structure_layouts layout JOIN spaces_structure_nodes node \
         ON node.id=layout.structure_node_id WHERE node.node_id=$1",
    )
    .bind(RESERVED)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(placed, (120.0, -40.0, 320.0, 180.0));
    let lineage: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM spaces_relations WHERE source_id=$1")
            .bind(RESERVED)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(lineage, 0);
}
