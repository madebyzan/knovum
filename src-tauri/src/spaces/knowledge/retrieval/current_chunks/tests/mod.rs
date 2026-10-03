use serde_json::Value;
use sqlx::sqlite::SqlitePoolOptions;

use super::filter;
use crate::spaces::knowledge::retrieval::generation::IndexGeneration;

fn generation() -> IndexGeneration {
    IndexGeneration {
        id: "generation".into(),
        embedding_spec_id: "spec".into(),
    }
}

async fn database() -> sqlx::SqlitePool {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("sqlite");
    crate::spaces::db::migrate(&pool)
        .await
        .expect("spaces migrations");
    pool
}

fn matched(chunk_id: &str, space_id: Option<&str>) -> Value {
    let mut value = serde_json::json!({
        "chunkId": chunk_id,
        "nodeId": "node",
        "nodeTitle": "Node",
        "sourceRevision": "rev",
        "text": "text",
        "score": 1.0,
    });
    if let Some(space_id) = space_id {
        value
            .as_object_mut()
            .unwrap()
            .insert("spaceId".to_owned(), Value::String(space_id.to_owned()));
    }
    value
}

#[tokio::test]
async fn drops_stale_matches_and_keeps_current_ones() {
    let pool = database().await;
    sqlx::query("INSERT INTO spaces(id,kind,title,created_at,updated_at,last_opened_at) VALUES ('space','space','Space',1,1,1)")
        .execute(&pool)
        .await
        .expect("space");
    sqlx::query("INSERT INTO spaces_nodes(id,space_id,kind,initial_x,initial_y,data,created_at,updated_at) VALUES ('node','space','note',0,0,'{\"kind\":\"note\",\"title\":\"Note\"}',1,1)")
        .execute(&pool)
        .await
        .expect("node");
    sqlx::query("INSERT INTO spaces_knowledge_chunks(id,space_id,node_id,origin,kind,source_revision,text,locator,metadata,created_at,updated_at) VALUES ('current','space','node','node','node','rev','text','{}','{}',1,1)")
        .execute(&pool)
        .await
        .expect("chunk");
    sqlx::query("INSERT INTO spaces_retrieval_generations(id,space_id,embedding_spec_id,status,created_at,activated_at) VALUES ('generation','space','spec','active',1,1)")
        .execute(&pool).await.expect("generation");
    sqlx::query("INSERT INTO spaces_chunk_embeddings(generation_id,chunk_id,source_revision,embedding_spec_id,dimensions,updated_at) VALUES ('generation','current','rev','spec',2,1)")
        .execute(&pool).await.expect("marker");

    let values = vec![matched("stale", None), matched("current", None)];
    let filtered = filter(&pool, "space", &generation(), values)
        .await
        .expect("filter");
    assert_eq!(filtered.len(), 1);
    assert_eq!(
        filtered[0].get("chunkId").and_then(Value::as_str),
        Some("current")
    );
}

#[tokio::test]
async fn uses_explicit_space_id_when_present() {
    let pool = database().await;
    sqlx::query("INSERT INTO spaces(id,kind,title,created_at,updated_at,last_opened_at) VALUES ('other','space','Space',1,1,1)")
        .execute(&pool)
        .await
        .expect("space");
    sqlx::query("INSERT INTO spaces_nodes(id,space_id,kind,initial_x,initial_y,data,created_at,updated_at) VALUES ('node','other','note',0,0,'{\"kind\":\"note\",\"title\":\"Note\"}',1,1)")
        .execute(&pool)
        .await
        .expect("node");
    sqlx::query("INSERT INTO spaces_knowledge_chunks(id,space_id,node_id,origin,kind,source_revision,text,locator,metadata,created_at,updated_at) VALUES ('chunk','other','node','node','node','rev','text','{}','{}',1,1)")
        .execute(&pool)
        .await
        .expect("chunk");
    sqlx::query("INSERT INTO spaces_retrieval_generations(id,space_id,embedding_spec_id,status,created_at,activated_at) VALUES ('generation','other','spec','active',1,1)")
        .execute(&pool).await.expect("generation");
    sqlx::query("INSERT INTO spaces_chunk_embeddings(generation_id,chunk_id,source_revision,embedding_spec_id,dimensions,updated_at) VALUES ('generation','chunk','rev','spec',2,1)")
        .execute(&pool).await.expect("marker");

    let values = vec![matched("chunk", Some("other"))];
    let filtered = filter(&pool, "wrong", &generation(), values)
        .await
        .expect("filter");
    assert_eq!(filtered.len(), 1);
}
