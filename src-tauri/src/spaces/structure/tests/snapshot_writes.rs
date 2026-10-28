use super::transactions::{database, request};
use crate::spaces::{
    db,
    structure::{commands::execute_in_transaction, repository},
};

#[tokio::test]
async fn sibling_reordering_preserves_the_active_order_constraint() {
    let pool = database().await;
    let mut tx = db::begin_write(&pool).await.unwrap();
    let mut snapshot: repository::SpaceStructureSnapshot = serde_json::from_value(serde_json::json!({
        "structure":[
            {"kind":"section","id":"first","parentId":null,"order":0,"locked":false,"title":"First"},
            {"kind":"section","id":"second","parentId":null,"order":1,"locked":false,"title":"Second"}
        ],
        "layouts":[
            {"kind":"section","structureNodeId":"first","position":{"x":0,"y":0},"size":{"width":300,"height":200},"collapsed":false},
            {"kind":"section","structureNodeId":"second","position":{"x":0,"y":0},"size":{"width":300,"height":200},"collapsed":false}
        ], "relation_styles":[]
    })).unwrap();
    repository::save_in_transaction(&mut tx, "space", 0, &mut snapshot)
        .await
        .unwrap();
    let mut reordered = snapshot;
    for node in &mut reordered.structure {
        if let crate::spaces::structure::StructureNode::Section { order, .. } = node {
            *order = 1 - *order;
        }
    }
    repository::save_in_transaction(&mut tx, "space", 1, &mut reordered)
        .await
        .unwrap();
    let (loaded, _) = repository::load_in_transaction(&mut tx, "space")
        .await
        .unwrap();
    assert_eq!(loaded.structure[0].id(), "second");
    assert_eq!(loaded.structure[1].id(), "first");
    tx.commit().await.unwrap();
}

#[tokio::test]
async fn unchanged_snapshot_does_not_rewrite_structure_or_layouts() {
    let pool = database().await;
    let mut tx = db::begin_write(&pool).await.unwrap();
    execute_in_transaction(&mut tx, request("Section"))
        .await
        .unwrap();
    tx.commit().await.unwrap();
    sqlx::query("CREATE TABLE snapshot_writes(kind TEXT); CREATE TRIGGER structure_written AFTER UPDATE ON spaces_structure_nodes BEGIN INSERT INTO snapshot_writes VALUES('structure'); END; CREATE TRIGGER layout_written AFTER UPDATE ON spaces_structure_layouts BEGIN INSERT INTO snapshot_writes VALUES('layout'); END;")
        .execute(&pool).await.unwrap();
    let mut tx = db::begin_write(&pool).await.unwrap();
    let (mut snapshot, revision) = repository::load_in_transaction(&mut tx, "space")
        .await
        .unwrap();
    repository::save_in_transaction(&mut tx, "space", revision, &mut snapshot)
        .await
        .unwrap();
    let writes: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM snapshot_writes")
        .fetch_one(&mut *tx)
        .await
        .unwrap();
    assert_eq!(writes, 0);
    let mut renamed = snapshot;
    if let crate::spaces::structure::StructureNode::Section { title, .. } =
        &mut renamed.structure[0]
    {
        *title = "Renamed".into();
    }
    repository::save_in_transaction(&mut tx, "space", revision + 1, &mut renamed)
        .await
        .unwrap();
    let writes: Vec<String> = sqlx::query_scalar("SELECT kind FROM snapshot_writes")
        .fetch_all(&mut *tx)
        .await
        .unwrap();
    assert_eq!(writes, ["structure"]);
    if let crate::spaces::structure::StructureLayout::Section { position, .. } =
        &mut renamed.layouts[0]
    {
        position.x = 50.0;
    }
    repository::save_in_transaction(&mut tx, "space", revision + 2, &mut renamed)
        .await
        .unwrap();
    let writes: Vec<String> = sqlx::query_scalar("SELECT kind FROM snapshot_writes")
        .fetch_all(&mut *tx)
        .await
        .unwrap();
    assert_eq!(writes, ["structure", "layout"]);
    let (loaded, _) = repository::load_in_transaction(&mut tx, "space")
        .await
        .unwrap();
    assert_eq!(
        serde_json::to_value(loaded.layouts).unwrap(),
        serde_json::to_value(renamed.layouts).unwrap()
    );
    tx.commit().await.unwrap();
}
