use sqlx::Row;

use super::super::invalidation;
use super::support::{database, seed_relation};

#[tokio::test]
async fn baseline_omits_removed_work_link_table() {
    let pool = database().await;
    let count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='spaces_work_links'",
    )
    .fetch_one(&pool)
    .await
    .expect("schema lookup");
    assert_eq!(count, 0);
}

#[test]
fn removed_work_link_patch_operation_is_rejected() {
    let decoded = serde_json::from_value::<Vec<super::super::change_commit::KnowledgeChange>>(
        serde_json::json!([{
            "op": "create-work-link",
            "tempId": "link",
            "nodeId": "node",
            "workKind": "task",
            "workId": "task",
            "role": "context"
        }]),
    );
    assert!(decoded.is_err());
}

#[tokio::test]
async fn relation_restore_preserves_active_status() {
    assert_restore_round_trip("active").await;
}

#[tokio::test]
async fn relation_restore_preserves_contested_status() {
    assert_restore_round_trip("contested").await;
}

async fn assert_restore_round_trip(initial_status: &str) {
    let pool = database().await;
    seed_relation(&pool, initial_status).await;
    let mut tx = pool.begin().await.expect("begin invalidation");
    assert!(
        invalidation::invalidate_relation(&mut tx, "relation", "human", None, 20)
            .await
            .expect("invalidate")
    );
    tx.commit().await.expect("commit invalidation");

    let row = sqlx::query(
        "SELECT status,status_before_invalidation FROM spaces_relations WHERE id='relation'",
    )
    .fetch_one(&pool)
    .await
    .expect("invalidated relation");
    assert_eq!(row.get::<String, _>("status"), "invalidated");
    assert_eq!(
        row.get::<String, _>("status_before_invalidation"),
        initial_status
    );

    let mut tx = pool.begin().await.expect("begin restore");
    let restored = invalidation::restore_relation(&mut tx, "relation", "human", None, 30)
        .await
        .expect("restore");
    tx.commit().await.expect("commit restore");
    assert_eq!(restored, initial_status);
    let row = sqlx::query(
        "SELECT status,status_before_invalidation FROM spaces_relations WHERE id='relation'",
    )
    .fetch_one(&pool)
    .await
    .expect("restored relation");
    assert_eq!(row.get::<String, _>("status"), initial_status);
    assert!(row
        .get::<Option<String>, _>("status_before_invalidation")
        .is_none());
}

#[tokio::test]
async fn invalidating_relation_invalidates_active_relation_provenance() {
    let pool = database().await;
    seed_relation(&pool, "active").await;
    sqlx::query(r#"INSERT INTO spaces_provenance(id,space_id,target_kind,target_id,source_node_id,locator,source_revision,quote,status,actor_kind,created_at,updated_at) VALUES ('prov','space','relation','relation','source','{"sourceRevision":"rev","location":{"kind":"pdf","page":1}}','rev','quote','active','human',1,1)"#)
        .execute(&pool).await.expect("provenance");
    let mut tx = pool.begin().await.expect("begin");
    invalidation::invalidate_relation(&mut tx, "relation", "human", None, 20)
        .await
        .expect("invalidate");
    tx.commit().await.expect("commit");
    let status =
        sqlx::query_scalar::<_, String>("SELECT status FROM spaces_provenance WHERE id='prov'")
            .fetch_one(&pool)
            .await
            .expect("provenance status");
    assert_eq!(status, "invalidated");
}
