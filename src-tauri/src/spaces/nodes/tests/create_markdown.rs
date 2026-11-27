use serde_json::json;

use crate::spaces::{
    authorization::{Principal, PrincipalKind},
    db,
    nodes::create_markdown_action,
    structure,
};

#[tokio::test]
async fn resolves_same_space_image_and_video_nodes_and_rejects_cross_space_nodes() {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    db::migrate(&pool).await.unwrap();
    sqlx::query("INSERT INTO spaces(id,kind,title,created_at,updated_at) VALUES('s1','space','S1',1,1),('s2','space','S2',1,1)")
        .execute(&pool)
        .await
        .unwrap();
    let mut init_tx = db::begin_write(&pool).await.unwrap();
    structure::repository::initialize_empty_in_transaction(&mut init_tx, "s1")
        .await
        .unwrap();
    structure::repository::initialize_empty_in_transaction(&mut init_tx, "s2")
        .await
        .unwrap();
    super::insert_source_in_transaction(
        &mut init_tx,
        "s1",
        "img1",
        "image",
        json!({"kind":"image","title":"Fox","fileName":"fox.png","assetPath":"/assets/fox.png"}),
    )
    .await
    .unwrap();
    super::insert_source_in_transaction(
        &mut init_tx,
        "s1",
        "vid1",
        "video",
        json!({"kind":"video","title":"Clip","fileName":"clip.mp4","assetPath":"/assets/clip.mp4"}),
    )
    .await
    .unwrap();
    super::insert_source_in_transaction(
        &mut init_tx,
        "s2",
        "img2",
        "image",
        json!({"kind":"image","title":"Other","fileName":"other.png","assetPath":"/assets/other.png"}),
    )
    .await
    .unwrap();
    init_tx.commit().await.unwrap();

    let principal = Principal {
        kind: PrincipalKind::BuiltinAgent,
        id: "task-1".into(),
    };
    let mut tx = db::begin_write(&pool).await.unwrap();
    let receipt = create_markdown_action::execute(
        &mut tx,
        &principal,
        "s1",
        &json!({
            "title": "补星星的小狐狸",
            "content": "# 补星星的小狐狸\n\n![小狐狸](knovum-node://img1)\n\n![片段](knovum-node://vid1)\n\n正文。"
        }),
        2,
    )
    .await
    .unwrap();
    let node_id = receipt["nodeId"].as_str().unwrap();
    let saved_text: String =
        sqlx::query_scalar("SELECT json_extract(data,'$.text') FROM spaces_nodes WHERE id=$1")
            .bind(node_id)
            .fetch_one(&mut *tx)
            .await
            .unwrap();
    assert!(saved_text.contains("%2Fassets%2Ffox.png"));
    assert!(saved_text.contains("%2Fassets%2Fclip.mp4"));
    assert!(!saved_text.contains("knovum-node://"));

    let cross_space = create_markdown_action::execute(
        &mut tx,
        &principal,
        "s1",
        &json!({
            "title": "Bad",
            "content": "![Other](knovum-node://img2)"
        }),
        3,
    )
    .await;
    assert!(cross_space.is_err());
}
