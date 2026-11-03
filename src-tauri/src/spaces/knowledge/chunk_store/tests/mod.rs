use super::validation::{validate_kind, validate_metadata};
use crate::spaces::knowledge::indexing::{KnowledgeChunkInput, MAX_KNOWLEDGE_PASSAGE_BYTES};
use serde_json::{json, Value};
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};

#[test]
fn metadata_rejects_nested_values() {
    assert!(
        validate_metadata(&json!({ "page": 2, "label": "ok", "flag": true, "empty": null }))
            .is_ok()
    );
    assert!(validate_metadata(&json!({ "nested": { "value": 1 } })).is_err());
    assert!(validate_metadata(&json!({ "list": [1, 2] })).is_err());
}

#[test]
fn kind_rejects_unknown_chunk_types() {
    assert!(validate_kind("text").is_ok());
    assert!(validate_kind("unknown").is_err());
    const { assert!(MAX_KNOWLEDGE_PASSAGE_BYTES < 64 * 1024) };
}

async fn document() -> SqlitePool {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    crate::spaces::db::migrate(&pool).await.unwrap();
    sqlx::query("INSERT INTO spaces(id,kind,title,created_at,updated_at) VALUES('space','space','Space',1,1)")
        .execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO spaces_nodes(id,space_id,kind,initial_x,initial_y,data,created_at,updated_at) VALUES('node','space','file',0,0,$1,1,1)")
        .bind(json!({ "kind": "file", "title": "Scan", "sourceRevision": "r1" }).to_string())
        .execute(&pool).await.unwrap();
    pool
}

fn passage(text: &str, role: &str) -> KnowledgeChunkInput {
    KnowledgeChunkInput {
        kind: "page".into(),
        text: text.into(),
        locator: json!({ "sourceRevision": "r1", "location": { "kind": "pdf", "page": 1 } }),
        metadata: Some(json!({ "role": role })),
    }
}

async fn passages(pool: &SqlitePool) -> Vec<(String, Value)> {
    sqlx::query_as::<_, (String, String)>(
        "SELECT text,metadata FROM spaces_knowledge_chunks WHERE origin='source' ORDER BY text",
    )
    .fetch_all(pool)
    .await
    .unwrap()
    .into_iter()
    .map(|(text, metadata)| (text, serde_json::from_str(&metadata).unwrap()))
    .collect()
}

#[tokio::test]
async fn replacing_a_role_keeps_the_passages_other_writers_put_in_the_origin() {
    let pool = document().await;
    let mut tx = pool.begin().await.unwrap();
    super::replace(
        &mut tx,
        "space",
        "node",
        "source",
        &[passage("Body", "text")],
        1,
    )
    .await
    .unwrap();
    for reading in ["First reading", "Second reading"] {
        let recognized = [passage(reading, "recognized_text")];
        super::replace_role(
            &mut tx,
            ("space", "node"),
            ("source", "recognized_text"),
            &recognized,
            2,
        )
        .await
        .unwrap();
    }
    let foreign = [passage("Body", "text")];
    let mislabelled = super::replace_role(
        &mut tx,
        ("space", "node"),
        ("source", "recognized_text"),
        &foreign,
        3,
    )
    .await;
    tx.commit().await.unwrap();

    assert!(
        mislabelled.is_err(),
        "a writer cannot replace another's passages"
    );
    assert_eq!(
        passages(&pool).await,
        [
            ("Body".to_owned(), json!({ "role": "text" })),
            (
                "Second reading".to_owned(),
                json!({ "role": "recognized_text" })
            ),
        ]
    );
}
