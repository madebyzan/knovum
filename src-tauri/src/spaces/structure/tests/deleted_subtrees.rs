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
