use crate::spaces::{
    assets::enrichment::jobs::revision::visual_revision,
    db, nodes,
    previews::GeneratedPreview,
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
        super::publish(&mut tx, "project", "revision", &output(), None)
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
    assert!(
        super::publish(&mut tx, "project", "revision", &changed, None)
            .await
            .is_err()
    );
}

#[tokio::test]
async fn a_dubbed_video_gets_its_own_cover_and_keeps_only_what_the_new_voice_leaves_true() {
    let pool = database().await;
    let mut tx = db::begin_write(&pool).await.unwrap();
    structure::repository::initialize_empty_in_transaction(&mut tx, "space")
        .await
        .unwrap();
    let mut source = json!({"kind":"video","title":"Interview","tags":["interview"],
        "assetPath":"/managed/source.mp4","fileName":"source.mp4","sourceRevision":"asset:1",
        "previewPath":"/previews/source.png","durationSeconds":2,
        "origin":{"kind":"web","url":"https://example.com/v","provider":"example","externalId":"v"},
        "mediaAnalysis":{"schemaVersion":1,"sourceRevision":"asset:1","durationMs":2_000,"waveform":[0.5]}});
    source["visualKnowledge"] = json!({
        "sourceRevision":visual_revision(&source).unwrap().unwrap(),"description":"Two people talking",
        "tags":[],"generatedAt":1,"executionSource":{"type":"model","provider":"openai","model":"vision"}
    });
    let request = serde_json::from_value(json!({
        "spaceId":"space","mutationId":"source-create","targetParentId":null,
        "nodes":[{"id":"source","kind":"video","initialX":0,"initialY":0,"width":320,"height":180,
            "sizingMode":"content","sourceChunks":[],"data":source}]
    }))
    .unwrap();
    nodes::tests::insert_many_in_transaction(&mut tx, request, None)
        .await
        .unwrap();
    sqlx::query("INSERT INTO spaces_speech_projects(id,space_id,source_node_id,project_type,title,created_at,updated_at) VALUES('project','space','source','video_dubbing','Interview dubbed',1,1)")
        .execute(&mut *tx).await.unwrap();
    let cover = GeneratedPreview {
        path: "/previews/dubbed.png".into(),
        width: 1280,
        height: 720,
    };
    let mut dubbed = output();
    dubbed.path = "/managed/dubbed.mp4".into();
    super::publish(&mut tx, "project", "revision", &dubbed, Some(cover))
        .await
        .unwrap();
    tx.commit().await.unwrap();
    let data: String =
        sqlx::query_scalar("SELECT data FROM spaces_nodes WHERE id='revision-output'")
            .fetch_one(&pool)
            .await
            .unwrap();
    let data: serde_json::Value = serde_json::from_str(&data).unwrap();
    assert_eq!(data["previewPath"], "/previews/dubbed.png");
    assert_eq!(data["intrinsicWidth"], 1280);
    assert_eq!(data["tags"], json!(["interview"]));
    assert_eq!(
        data["visualKnowledge"]["sourceRevision"],
        visual_revision(&data).unwrap().unwrap().as_str()
    );
    assert!(data.get("origin").is_none());
    assert!(
        data.get("mediaAnalysis").is_none(),
        "a new voice is measured anew"
    );
    let queued: Vec<String> = sqlx::query_scalar(
        "SELECT job_type FROM spaces_enrichment_jobs WHERE node_id='revision-output'",
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(queued, ["analyze_media"]);
}

#[tokio::test]
async fn publication_is_atomic_and_a_deleted_source_cannot_receive_outputs() {
    let pool = fixture().await;
    let mut tx = db::begin_write(&pool).await.unwrap();
    super::publish(&mut tx, "project", "revision", &output(), None)
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
    assert!(
        super::publish(&mut tx, "project", "revision", &output(), None)
            .await
            .is_err()
    );
}
