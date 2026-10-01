use serde_json::json;

#[tokio::test]
async fn user_edits_receive_a_rust_owned_revision() {
    let pool = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
    crate::spaces::db::migrate(&pool).await.unwrap();
    let mut tx = pool.begin().await.unwrap();
    let mut data = json!({"kind":"markdown","title":"Note","text":"Updated text","knowledgeRevision":"obsolete-ui-hash"});
    crate::spaces::knowledge::contracts::normalize_node(&mut tx, "space", "markdown", &mut data)
        .await
        .unwrap();
    assert_eq!(
        data["knowledgeRevision"].as_str(),
        crate::spaces::knowledge::indexing::revision::authoritative(&data)
            .unwrap()
            .as_deref()
    );
    assert_ne!(data["knowledgeRevision"], "obsolete-ui-hash");
}

#[tokio::test]
async fn resetting_image_edits_replaces_the_visual_job_through_the_node_update_pipeline() {
    use crate::spaces::{assets::enrichment::jobs, nodes::contracts::NodeContentChange};
    let pool = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
    crate::spaces::db::migrate(&pool).await.unwrap();
    let adjustments: serde_json::Map<String, serde_json::Value> = [
        "exposure",
        "contrast",
        "highlights",
        "shadows",
        "whites",
        "blacks",
        "saturation",
        "warmth",
        "sharpness",
        "vignette",
        "vibrance",
        "tint",
        "clarity",
        "grain",
        "softLight",
        "glow",
    ]
    .into_iter()
    .map(|key| (key.to_owned(), json!(0.0)))
    .collect();
    let mut before = json!({
        "kind":"image", "title":"Picture", "sourceRevision":"asset-1",
        "assetPath":"/assets/picture.png", "fileName":"picture.png",
        "imageEdit":{
            "version":6, "crop":{"x":0.0,"y":0.0,"width":1.0,"height":1.0},
            "rotation":90.0,"flipHorizontal":false,"flipVertical":false,
            "adjustments":adjustments
        }
    });
    crate::spaces::knowledge::indexing::revision::apply(&mut before).unwrap();
    sqlx::query("INSERT INTO spaces(id,kind,title,created_at,updated_at) VALUES('space','space','Space',1,1)")
        .execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO spaces_nodes(id,space_id,kind,initial_x,initial_y,data,created_at,updated_at) VALUES('node','space','image',0,0,$1,1,1)")
        .bind(before.to_string()).execute(&pool).await.unwrap();
    let mut tx = pool.begin().await.unwrap();
    jobs::intent::synchronize_for_node(&mut tx, "space", "node", &before, 1, None)
        .await
        .unwrap();
    let mut after = before.clone();
    after.as_object_mut().unwrap().remove("imageEdit");
    crate::spaces::knowledge::contracts::normalize_node(&mut tx, "space", "image", &mut after)
        .await
        .unwrap();
    super::update_many_in_transaction(
        &mut tx,
        "space",
        &[NodeContentChange {
            id: "node".into(),
            expected_data: before,
            data: after.clone(),
            source_chunks: None,
        }],
        "human",
        2,
        None,
    )
    .await
    .unwrap();
    tx.commit().await.unwrap();
    let cancelled: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM spaces_enrichment_jobs WHERE status='cancelled'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(cancelled, 1);
    let pending: Vec<String> = sqlx::query_scalar("SELECT source_revision FROM spaces_enrichment_jobs WHERE job_type='analyze_visual' AND status='pending'")
        .fetch_all(&pool).await.unwrap();
    assert_eq!(
        pending,
        vec![jobs::revision::visual_revision(&after).unwrap().unwrap()]
    );
}
