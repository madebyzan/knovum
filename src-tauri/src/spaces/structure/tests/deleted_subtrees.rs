use super::transactions::database;
use crate::spaces::{db, structure::repository};

#[tokio::test]
async fn restores_one_large_subtree_and_retains_the_other_root() {
    let pool = database().await;
    let mut tx = db::begin_write(&pool).await.unwrap();
    sqlx::query("INSERT INTO spaces_deleted_structure_batches(id,space_id,root_structure_node_ids,deleted_at) VALUES('batch','space','[\"root\",\"other\"]',1)")
        .execute(&mut *tx).await.unwrap();
    for index in 0..1002 {
        let (id, parent, order) = match index {
            0 => ("root".into(), None, 0),
            1 => ("other".into(), None, 1),
            _ => (format!("child-{index}"), Some("root"), index - 2),
        };
        let data = serde_json::json!({"kind":"group","id":id,"parentId":parent,"order":order,"locked":false,"title":"Group"});
        let layout = serde_json::json!({"kind":"group","structureNodeId":id,"transform":{"x":0,"y":0,"rotation":0,"scale":1}});
        sqlx::query("INSERT INTO spaces_deleted_structure_nodes(batch_id,structure_node_id,data) VALUES('batch',$1,$2)")
            .bind(&id).bind(data.to_string()).execute(&mut *tx).await.unwrap();
        sqlx::query("INSERT INTO spaces_deleted_structure_layouts(batch_id,structure_node_id,data) VALUES('batch',$1,$2)")
            .bind(&id).bind(layout.to_string()).execute(&mut *tx).await.unwrap();
    }
    let restored = repository::prepare_restore_root(&mut tx, "space", "batch", "root", "partial")
        .await
        .unwrap();
    assert_eq!(restored.structure.len(), 1001);
    assert_eq!(restored.layouts.len(), 1001);
    assert!(!restored.structure.iter().any(|node| node.id() == "other"));
    let remaining: Vec<String> = sqlx::query_scalar(
        "SELECT structure_node_id FROM spaces_deleted_structure_nodes WHERE batch_id='batch'",
    )
    .fetch_all(&mut *tx)
    .await
    .unwrap();
    assert_eq!(remaining, ["other"]);
    tx.rollback().await.unwrap();
}

#[tokio::test]
async fn restores_parent_and_child_tombstones_atomically() {
    let pool = database().await;
    let snapshot: repository::SpaceStructureSnapshot = serde_json::from_value(serde_json::json!({
        "structure":[
            {"kind":"group","id":"root","parentId":null,"order":0,"locked":false,"title":"Root"},
            {"kind":"group","id":"child","parentId":"root","order":0,"locked":false,"title":"Child"}
        ],
        "layouts":[
            {"kind":"group","structureNodeId":"root","transform":{"x":0,"y":0,"rotation":0,"scale":1}},
            {"kind":"group","structureNodeId":"child","transform":{"x":0,"y":0,"rotation":0,"scale":1}}
        ], "relation_styles":[]
    })).unwrap();
    let mut tx = db::begin_write(&pool).await.unwrap();
    repository::save_in_transaction(&mut tx, "space", 0, &snapshot)
        .await
        .unwrap();
    sqlx::query("UPDATE spaces_structure_nodes SET deleted_at=1 WHERE space_id='space'")
        .execute(&mut *tx)
        .await
        .unwrap();
    sqlx::query("INSERT INTO spaces_deleted_structure_batches(id,space_id,root_structure_node_ids,deleted_at) VALUES('batch','space','[\"root\"]',1)")
        .execute(&mut *tx).await.unwrap();
    for node in &snapshot.structure {
        sqlx::query("INSERT INTO spaces_deleted_structure_nodes(batch_id,structure_node_id,data) VALUES('batch',$1,$2)")
            .bind(node.id()).bind(serde_json::to_string(node).unwrap()).execute(&mut *tx).await.unwrap();
    }
    for layout in &snapshot.layouts {
        sqlx::query("INSERT INTO spaces_deleted_structure_layouts(batch_id,structure_node_id,data) VALUES('batch',$1,$2)")
            .bind(layout.structure_node_id()).bind(serde_json::to_string(layout).unwrap()).execute(&mut *tx).await.unwrap();
    }
    tx.commit().await.unwrap();

    let mut tx = db::begin_write(&pool).await.unwrap();
    repository::prepare_restore_batch(&mut tx, "space", "batch")
        .await
        .unwrap();
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM spaces_structure_nodes")
        .fetch_one(&mut *tx)
        .await
        .unwrap();
    assert_eq!(count, 0);
    tx.rollback().await.unwrap();
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM spaces_structure_nodes WHERE deleted_at IS NOT NULL",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(count, 2);

    let mut tx = db::begin_write(&pool).await.unwrap();
    let restored = repository::prepare_restore_batch(&mut tx, "space", "batch")
        .await
        .unwrap();
    let snapshot = repository::SpaceStructureSnapshot {
        structure: restored.structure,
        layouts: restored.layouts,
        relation_styles: restored.relation_styles,
    };
    repository::save_in_transaction(&mut tx, "space", 1, &snapshot)
        .await
        .unwrap();
    repository::mark_restored(&mut tx, "space", &restored.batch_ids, 2)
        .await
        .unwrap();
    tx.commit().await.unwrap();
    let count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM spaces_structure_nodes WHERE deleted_at IS NULL")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(count, 2);
}
