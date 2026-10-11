use super::{EditorResourceProtection, SpaceEditorResource};
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};

async fn database() -> SqlitePool {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    sqlx::raw_sql("CREATE TABLE spaces_nodes(id TEXT PRIMARY KEY,space_id TEXT,deleted_at INTEGER);
        CREATE TABLE spaces_structure_nodes(id TEXT PRIMARY KEY,node_id TEXT,space_id TEXT,deleted_at INTEGER);
        INSERT INTO spaces_nodes VALUES('node','space',NULL),('other','space',NULL);
        INSERT INTO spaces_structure_nodes VALUES('content','node','space',NULL),('other','other','space',NULL);")
        .execute(&pool).await.unwrap();
    pool
}

fn resource() -> SpaceEditorResource {
    SpaceEditorResource {
        space_id: "space".into(),
        node_id: "node".into(),
    }
}

#[tokio::test]
async fn rejected_deletion_rolls_back_the_complete_batch() {
    let pool = database().await;
    let mut tx = pool.begin().await.unwrap();
    let protection = EditorResourceProtection::capture(&mut tx, &[resource()])
        .await
        .unwrap();
    sqlx::query("UPDATE spaces_nodes SET deleted_at=1")
        .execute(&mut *tx)
        .await
        .unwrap();
    let error = protection.ensure_preserved(&mut tx).await.unwrap_err();
    assert_eq!(error.code, "space_editor_resource_protected");
    tx.rollback().await.unwrap();
    let active: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM spaces_nodes WHERE deleted_at IS NULL")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(active, 2);
}

#[tokio::test]
async fn rejects_lost_structure_and_cross_space_ownership() {
    let pool = database().await;
    for mutation in [
        "DELETE FROM spaces_structure_nodes WHERE node_id='node'",
        "UPDATE spaces_nodes SET space_id='other-space' WHERE id='node'",
        "UPDATE spaces_structure_nodes SET deleted_at=1 WHERE node_id='node'",
    ] {
        let mut tx = pool.begin().await.unwrap();
        let protection = EditorResourceProtection::capture(&mut tx, &[resource()])
            .await
            .unwrap();
        sqlx::query(mutation).execute(&mut *tx).await.unwrap();
        assert_eq!(
            protection.ensure_preserved(&mut tx).await.unwrap_err().code,
            "space_editor_resource_protected"
        );
        tx.rollback().await.unwrap();
    }
}

#[tokio::test]
async fn unrelated_changes_and_container_identity_changes_preserve_protection() {
    let pool = database().await;
    let mut tx = pool.begin().await.unwrap();
    let protection = EditorResourceProtection::capture(&mut tx, &[resource()])
        .await
        .unwrap();
    sqlx::raw_sql("UPDATE spaces_nodes SET deleted_at=1 WHERE id='other'; UPDATE spaces_structure_nodes SET id='new-placement' WHERE node_id='node';")
        .execute(&mut *tx).await.unwrap();
    protection.ensure_preserved(&mut tx).await.unwrap();
    tx.commit().await.unwrap();
}

#[tokio::test]
async fn an_already_missing_resource_does_not_block_unrelated_work() {
    let pool = database().await;
    sqlx::query("DELETE FROM spaces_nodes WHERE id='node'")
        .execute(&pool)
        .await
        .unwrap();
    let mut tx = pool.begin().await.unwrap();
    let protection = EditorResourceProtection::capture(&mut tx, &[resource()])
        .await
        .unwrap();
    protection.ensure_preserved(&mut tx).await.unwrap();
}

#[tokio::test]
async fn rejects_malformed_ipc_resource_identities() {
    let pool = database().await;
    let mut tx = pool.begin().await.unwrap();
    let bad = SpaceEditorResource {
        space_id: String::new(),
        node_id: "node".into(),
    };
    assert!(EditorResourceProtection::capture(&mut tx, &[bad])
        .await
        .is_err());
    assert!(serde_json::from_str::<SpaceEditorResource>(
        r#"{"spaceId":"space","nodeId":"node","locked":true}"#
    )
    .is_err());
}
