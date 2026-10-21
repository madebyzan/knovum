use serde_json::{json, Value};
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};

use super::{cite_answer, markers, number_passages, Reading};

async fn database() -> SqlitePool {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    crate::spaces::db::migrate(&pool).await.unwrap();
    sqlx::raw_sql(
        r#"INSERT INTO spaces(id,kind,title,created_at,updated_at) VALUES('space','space','Space',1,1);
        INSERT INTO spaces_nodes(id,space_id,kind,initial_x,initial_y,data,created_at,updated_at)
          VALUES('node','space','file',0,0,'{"title":"Plan.docx"}',1,1);
        INSERT INTO spaces_knowledge_chunks(id,space_id,node_id,origin,kind,source_revision,text,locator,metadata,created_at,updated_at)
          VALUES('first','space','node','node','node','rev','First passage','{"page":1}','{}',1,1),
                ('second','space','node','node','node','rev','Second passage','{"page":2}','{}',1,1);
        INSERT INTO spaces_runs(id,scope_kind,home_space_id,principal_kind,principal_id,kind,state,policy_snapshot,input_snapshot,budget,created_at,updated_at)
          VALUES('run','space','space','builtin_agent','task','builtin_agent','running','{}','{}','{}',1,1),
                ('other','space','space','builtin_agent','task','builtin_agent','running','{}','{}','{}',1,1);"#,
    )
    .execute(&pool)
    .await
    .unwrap();
    pool
}

fn search(chunks: &[&str]) -> Value {
    Value::Array(
        chunks
            .iter()
            .map(|chunk| json!({"nodeId": "node", "chunkId": chunk, "sourceRevision": "rev"}))
            .collect(),
    )
}

#[tokio::test]
async fn a_run_numbers_each_passage_once_across_searches_and_node_reads() {
    let pool = database().await;
    let mut connection = pool.acquire().await.unwrap();
    let mut first = search(&["second", "first"]);
    number_passages(&mut connection, "run", Reading::Search, &mut first)
        .await
        .unwrap();
    let mut node = json!({"id": "node", "chunks": [
        {"id": "first", "sourceRevision": "rev"},
        {"id": "second", "sourceRevision": "rev"},
    ]});
    number_passages(&mut connection, "run", Reading::Node, &mut node)
        .await
        .unwrap();
    let mut other = search(&["first"]);
    number_passages(&mut connection, "other", Reading::Search, &mut other)
        .await
        .unwrap();
    assert_eq!(first[0]["cite"], "C1");
    assert_eq!(first[1]["cite"], "C2");
    assert_eq!(node["chunks"][0]["cite"], "C2");
    assert_eq!(node["chunks"][1]["cite"], "C1");
    assert_eq!(other[0]["cite"], "C1");
}

#[tokio::test]
async fn a_passage_that_is_no_longer_current_gets_no_cite() {
    let pool = database().await;
    let mut connection = pool.acquire().await.unwrap();
    let mut stale = json!([{"nodeId": "node", "chunkId": "first", "sourceRevision": "old"}]);
    number_passages(&mut connection, "run", Reading::Search, &mut stale)
        .await
        .unwrap();
    assert!(stale[0].get("cite").is_none());
}

#[tokio::test]
async fn an_answer_resolves_only_issued_markers_and_detaches_changed_sources() {
    let pool = database().await;
    let mut connection = pool.acquire().await.unwrap();
    let mut read = search(&["first", "second"]);
    number_passages(&mut connection, "run", Reading::Search, &mut read)
        .await
        .unwrap();
    sqlx::query("UPDATE spaces_knowledge_chunks SET source_revision='next' WHERE id='second'")
        .execute(&mut *connection)
        .await
        .unwrap();
    let citations = cite_answer(&mut connection, "run", "B [C2], A [C1][C9] and [C1] again.")
        .await
        .unwrap();
    assert_eq!(
        citations,
        vec![
            json!({"index": 1, "state": "current", "chunkId": "first", "nodeId": "node",
                "nodeTitle": "Plan.docx", "sourceRevision": "rev", "locator": {"page": 1},
                "quote": "First passage"}),
            json!({"index": 2, "state": "detached", "nodeTitle": "Plan.docx",
                "sourceRevision": "rev", "locator": {"page": 2}, "quote": "Second passage"}),
        ]
    );
}

#[test]
fn markers_are_bracketed_c_numbers() {
    assert_eq!(
        markers("[C3] [C12][C3] [C] [c4] C5 [C6 [C0] ok [C7]"),
        vec![3, 12, 7]
    );
}
