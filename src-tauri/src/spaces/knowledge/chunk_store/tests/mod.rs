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

fn media_passage(
    role: &str,
    kind: &str,
    text: &str,
    start_ms: i64,
    end_ms: i64,
    metadata: Value,
) -> KnowledgeChunkInput {
    let mut meta = metadata.as_object().cloned().unwrap_or_default();
    meta.insert("role".into(), Value::String(role.into()));
    KnowledgeChunkInput {
        kind: kind.into(),
        text: text.into(),
        locator: json!({
            "sourceRevision": "r1",
            "location": { "kind": "media", "startMs": start_ms, "endMs": end_ms }
        }),
        metadata: Some(Value::Object(meta)),
    }
}

#[tokio::test]
async fn replacing_media_chunks_preserves_unchanged_transcript_citations_and_detaches_changed_segments(
) {
    let pool = document().await;
    sqlx::query("INSERT INTO spaces_conversations(id,space_id,title,created_at,updated_at,active_branch_id) VALUES('conv','space','Chat',1,1,'conv')")
        .execute(&pool)
        .await
        .unwrap();

    let seg1 = media_passage(
        "transcript",
        "transcript",
        "I auditioned when I was nine",
        1000,
        3000,
        json!({ "transcriptRevision": 1, "segmentId": "s1" }),
    );
    let seg2 = media_passage(
        "transcript",
        "transcript",
        "Quite horrible, to be honest.",
        3000,
        5000,
        json!({ "transcriptRevision": 1, "segmentId": "s2" }),
    );
    let mut tx = pool.begin().await.unwrap();
    super::replace(
        &mut tx,
        "space",
        "node",
        "media",
        &[seg1.clone(), seg2.clone()],
        1,
    )
    .await
    .unwrap();
    tx.commit().await.unwrap();

    let initial_ids: Vec<(String, String)> = sqlx::query_as(
        "SELECT text, id FROM spaces_knowledge_chunks WHERE origin='media' ORDER BY created_at, rowid",
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    let seg1_id = initial_ids[0].1.clone();
    let seg2_id = initial_ids[1].1.clone();

    sqlx::query("INSERT INTO spaces_conversation_messages(id,conversation_id,branch_id,role,content,created_at) VALUES('msg','conv','conv','assistant',$1,1)")
        .bind(
            json!({
                "parts": [{ "type": "text", "text": "Cited [C1] and [C2]." }],
                "citations": [
                    { "index": 1, "state": "current", "nodeId": "node", "chunkId": seg1_id, "sourceRevision": "r1", "nodeTitle": "Scan", "locator": seg1.locator, "quote": seg1.text },
                    { "index": 2, "state": "current", "nodeId": "node", "chunkId": seg2_id, "sourceRevision": "r1", "nodeTitle": "Scan", "locator": seg2.locator, "quote": seg2.text }
                ]
            })
            .to_string(),
        )
        .execute(&pool)
        .await
        .unwrap();

    // Generating summary + chapters + speaker diarization bumps transcriptRevision without changing text/timings.
    let summary = media_passage("summary", "text", "Interview overview", 0, 5000, json!({}));
    let seg1_diarized = media_passage(
        "transcript",
        "transcript",
        "I auditioned when I was nine",
        1000,
        3000,
        json!({ "transcriptRevision": 2, "segmentId": "s1" }),
    );
    let seg2_diarized = media_passage(
        "transcript",
        "transcript",
        "Quite horrible, to be honest.",
        3000,
        5000,
        json!({ "transcriptRevision": 2, "segmentId": "s2" }),
    );
    let mut tx = pool.begin().await.unwrap();
    super::replace(
        &mut tx,
        "space",
        "node",
        "media",
        &[summary.clone(), seg1_diarized.clone(), seg2_diarized],
        2,
    )
    .await
    .unwrap();
    tx.commit().await.unwrap();

    let after_enrichment: String =
        sqlx::query_scalar("SELECT content FROM spaces_conversation_messages WHERE id='msg'")
            .fetch_one(&pool)
            .await
            .unwrap();
    let after_enrichment: Value = serde_json::from_str(&after_enrichment).unwrap();
    assert_eq!(after_enrichment["citations"][0]["state"], "current");
    assert_eq!(after_enrichment["citations"][1]["state"], "current");
    let preserved_meta: String =
        sqlx::query_scalar("SELECT metadata FROM spaces_knowledge_chunks WHERE id=$1")
            .bind(&seg1_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(
        serde_json::from_str::<Value>(&preserved_meta).unwrap()["transcriptRevision"],
        2
    );

    // Editing only segment 2's text detaches only segment 2's citation while segment 1 stays current.
    let seg2_edited = media_passage(
        "transcript",
        "transcript",
        "Actually quite lovely.",
        3000,
        5000,
        json!({ "transcriptRevision": 3, "segmentId": "s2" }),
    );
    let mut tx = pool.begin().await.unwrap();
    super::replace(
        &mut tx,
        "space",
        "node",
        "media",
        &[summary, seg1_diarized, seg2_edited],
        3,
    )
    .await
    .unwrap();
    tx.commit().await.unwrap();

    let after_edit: String =
        sqlx::query_scalar("SELECT content FROM spaces_conversation_messages WHERE id='msg'")
            .fetch_one(&pool)
            .await
            .unwrap();
    let after_edit: Value = serde_json::from_str(&after_edit).unwrap();
    assert_eq!(after_edit["citations"][0]["state"], "current");
    assert_eq!(after_edit["citations"][0]["chunkId"], seg1_id);
    assert_eq!(after_edit["citations"][1]["state"], "detached");
}
