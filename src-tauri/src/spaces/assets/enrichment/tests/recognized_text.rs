use serde_json::json;
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};

use super::*;
use crate::spaces::knowledge::{
    chunk_store,
    indexing::{self, KnowledgeChunkInput},
};

async fn document(revision: &str) -> SqlitePool {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    crate::spaces::db::migrate(&pool).await.unwrap();
    sqlx::query("INSERT INTO spaces(id,kind,title,created_at,updated_at) VALUES('space','space','Space',1,1)")
        .execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO spaces_nodes(id,space_id,kind,initial_x,initial_y,data,created_at,updated_at) VALUES('node','space','file',0,0,$1,1,1)")
        .bind(json!({ "kind": "file", "title": "Brief", "sourceRevision": revision }).to_string())
        .execute(&pool).await.unwrap();
    pool
}

fn read(text: &str, metadata: serde_json::Value) -> Vec<KnowledgeChunkInput> {
    let location = json!({ "kind": "pdf", "page": 3 });
    indexing::chunks("page", text, "r1", location, metadata)
        .into_iter()
        .map(Into::into)
        .collect()
}

#[tokio::test]
async fn each_picture_reads_back_as_its_own_text_in_reading_order() {
    let pool = document("r1").await;
    let mut tx = pool.begin().await.unwrap();
    let body = read("Summary", json!({ "role": "text" }));
    chunk_store::replace(&mut tx, "space", "node", "source", &body, 1)
        .await
        .unwrap();
    let long_scan = format!("{}\n{}", "Clause 4. ".repeat(400), "Clause 7.");
    let mut recognized = read(
        &long_scan,
        json!({ "role": RECOGNIZED_TEXT_ROLE, "resourceId": "scan" }),
    );
    assert!(recognized.len() > 1, "a long reading spans passages");
    recognized.extend(read(
        "Mill Lane",
        json!({ "role": RECOGNIZED_TEXT_ROLE, "resourceId": "photo" }),
    ));
    chunk_store::replace_role(
        &mut tx,
        ("space", "node"),
        ("source", RECOGNIZED_TEXT_ROLE),
        &recognized,
        2,
    )
    .await
    .unwrap();
    tx.commit().await.unwrap();

    let readings = readings(&pool, "space", "node").await.unwrap();
    assert_eq!(readings.len(), 2, "the body passage is not a reading");
    assert_eq!(readings[0].resource_id.as_deref(), Some("scan"));
    assert!(readings[0].text.starts_with("Clause 4.") && readings[0].text.ends_with("Clause 7."));
    assert_eq!(
        readings[1],
        RecognizedTextReading {
            resource_id: Some("photo".into()),
            text: "Mill Lane".into(),
        }
    );
}

#[tokio::test]
async fn a_reading_of_an_older_source_is_not_current() {
    let pool = document("r1").await;
    let mut tx = pool.begin().await.unwrap();
    let recognized = read("Old scan", json!({ "role": RECOGNIZED_TEXT_ROLE }));
    chunk_store::replace_role(
        &mut tx,
        ("space", "node"),
        ("source", RECOGNIZED_TEXT_ROLE),
        &recognized,
        1,
    )
    .await
    .unwrap();
    sqlx::query("UPDATE spaces_nodes SET data=json_set(data,'$.sourceRevision','r2')")
        .execute(&mut *tx)
        .await
        .unwrap();
    tx.commit().await.unwrap();
    assert!(readings(&pool, "space", "node").await.unwrap().is_empty());
}
