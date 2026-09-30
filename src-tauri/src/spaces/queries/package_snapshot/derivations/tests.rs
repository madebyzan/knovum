use super::*;
use serde_json::json;

#[tokio::test]
async fn exports_real_execution_sources_for_document_image_and_video() {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    db::migrate(&pool).await.unwrap();
    sqlx::query("INSERT INTO spaces(id,kind,title,created_at,updated_at) VALUES('space','space','Space',1,1)").execute(&pool).await.unwrap();
    for (id, action) in [
        ("document", "draft-document"),
        ("image", "generate-visualization"),
        ("video", "generate-video"),
    ] {
        sqlx::query("INSERT INTO spaces_nodes(id,space_id,kind,initial_x,initial_y,data,created_at,updated_at) VALUES($1,'space','markdown',0,0,$2,1,1)")
            .bind(id).bind(json!({"kind":"markdown","title":"Record","text":"Body"}).to_string()).execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO spaces_knowledge_derivations(id,space_id,output_node_id,action_kind,action_version,engine,status,error,created_at,updated_at,completed_at) VALUES($1,'space',$1,$2,1,'desktop-agent:codex','completed','',1,1,1)")
            .bind(id).bind(action).execute(&pool).await.unwrap();
    }
    let source = json!({"type":"agent","runtimeKind":"codex"});
    sqlx::query("INSERT INTO spaces_grounded_documents(derivation_id,citations,execution_source) VALUES('document','[]',$1)").bind(source.to_string()).execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO spaces_generated_images(derivation_id,intent,style,aspect_ratio,execution_source) VALUES('image','Explain','','1:1',$1)").bind(source.to_string()).execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO spaces_generated_videos(derivation_id,prompt,aspect_ratio,duration_seconds,execution_source,operation_id) VALUES('video','Explain','16:9',10,$1,NULL)").bind(source.to_string()).execute(&pool).await.unwrap();
    let mut tx = pool.begin().await.unwrap();
    let rows = load(&mut tx, "space").await.unwrap();
    assert_eq!(rows.len(), 3);
    for (id, field) in [
        ("document", "groundedDocument"),
        ("image", "generatedImage"),
        ("video", "generatedVideo"),
    ] {
        let row = rows.iter().find(|row| row["id"] == id).unwrap();
        assert_eq!(row[field]["executionSource"], source);
        assert!(row[field].get("provider").is_none());
        assert!(row[field].get("model").is_none());
    }
    assert!(
        rows.iter().find(|row| row["id"] == "video").unwrap()["generatedVideo"]["operationId"]
            .is_null()
    );
}
