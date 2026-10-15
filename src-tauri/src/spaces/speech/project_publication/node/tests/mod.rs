use crate::spaces::{
    db, nodes,
    speech::{contracts::SpeechAudioOutput, tests::database},
    structure,
};
use serde_json::json;

async fn fixture() -> sqlx::SqlitePool {
    let pool = database().await;
    let mut tx = db::begin_write(&pool).await.unwrap();
    structure::repository::initialize_empty_in_transaction(&mut tx, "space")
        .await
        .unwrap();
    let request = serde_json::from_value(json!({
        "spaceId":"space","mutationId":"source-create","targetParentId":null,
        "nodes":[{"id":"source","kind":"note","initialX":0,"initialY":0,"width":320,"height":100,
            "sizingMode":"content","sourceChunks":[],"data":{"kind":"note","title":"Source",
                "plainText":"Hello","tiptapContent":{"type":"doc","content":[{"type":"paragraph"}]}}}]
    })).unwrap();
    nodes::tests::insert_many_in_transaction(&mut tx, request, None)
        .await
        .unwrap();
    sqlx::query("INSERT INTO spaces_speech_projects(id,space_id,source_node_id,project_type,title,created_at,updated_at) VALUES('project','space','source','short_dubbing','Narration',1,1)")
        .execute(&mut *tx).await.unwrap();
    tx.commit().await.unwrap();
    pool
}

fn output() -> SpeechAudioOutput {
    SpeechAudioOutput {
        path: "/managed/narration.wav".into(),
        sha256: "a".repeat(64),
        sample_rate: 24000,
        sample_count: 48000,
    }
}

#[tokio::test]
async fn publication_creates_one_audio_node_next_to_source_and_retries_are_idempotent() {
    let pool = fixture().await;
    for _ in 0..2 {
        let mut tx = db::begin_write(&pool).await.unwrap();
        super::publish(&mut tx, "project", "revision", &output())
            .await
            .unwrap();
        tx.commit().await.unwrap();
    }
    let (kind, data): (String, String) =
        sqlx::query_as("SELECT kind,data FROM spaces_nodes WHERE id='revision-output'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(kind, "audio");
    let data: serde_json::Value = serde_json::from_str(&data).unwrap();
    assert_eq!(data["assetPath"], output().path);
    assert_eq!(data["durationSeconds"].as_u64(), Some(2));
    let (output_id, source_id, kind): (String, String, String) = sqlx::query_as(
        "SELECT source_id,target_id,kind FROM spaces_relations WHERE source_id='revision-output'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(
        (output_id.as_str(), source_id.as_str(), kind.as_str()),
        ("revision-output", "source", "derives-from")
    );
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM spaces_relations")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 1);

    let mut tx = db::begin_write(&pool).await.unwrap();
    let (snapshot, revision) = structure::repository::load_in_transaction(&mut tx, "space")
        .await
        .unwrap();
    assert_eq!(snapshot.structure.len(), 2);
    assert_eq!(revision, 2);
    let mut changed = output();
    changed.sha256 = "b".repeat(64);
    assert!(super::publish(&mut tx, "project", "revision", &changed)
        .await
        .is_err());
}

#[tokio::test]
async fn publication_is_atomic_and_a_deleted_source_cannot_receive_outputs() {
    let pool = fixture().await;
    let mut tx = db::begin_write(&pool).await.unwrap();
    super::publish(&mut tx, "project", "revision", &output())
        .await
        .unwrap();
    tx.rollback().await.unwrap();
    let count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM spaces_nodes WHERE id='revision-output'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(count, 0);
    sqlx::query("UPDATE spaces_nodes SET deleted_at=2 WHERE id='source'")
        .execute(&pool)
        .await
        .unwrap();
    let mut tx = db::begin_write(&pool).await.unwrap();
    assert!(super::publish(&mut tx, "project", "revision", &output())
        .await
        .is_err());
}
