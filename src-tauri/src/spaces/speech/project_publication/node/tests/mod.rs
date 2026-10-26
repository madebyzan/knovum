use serde_json::json;

use crate::spaces::{
    assets::enrichment::jobs::revision::visual_revision,
    db,
    derivation::pending_output::{self, PendingOutput},
    nodes,
    previews::GeneratedPreview,
    speech::{contracts::SpeechAudioOutput, tests::database},
    structure::{self, Size},
};

/// A Space holding one source node, the Run that will speak it, and the node reserved for that
/// Run's output — the state publication always finds.
async fn fixture(
    project_type: &str,
    source: serde_json::Value,
    output_kind: &'static str,
) -> sqlx::SqlitePool {
    let pool = database().await;
    let mut tx = db::begin_write(&pool).await.unwrap();
    structure::repository::initialize_empty_in_transaction(&mut tx, "space")
        .await
        .unwrap();
    sqlx::query("INSERT INTO spaces_runs(id,scope_kind,home_space_id,principal_kind,principal_id,kind,state,execution_epoch,policy_snapshot,input_snapshot,budget,created_at,updated_at) VALUES('run','space','space','user','local-user','workflow','running',0,'{}','{}','{}',1,1)")
        .execute(&mut *tx).await.unwrap();
    let kind = source["kind"].as_str().unwrap().to_owned();
    nodes::tests::insert_source_in_transaction(&mut tx, "space", "source", &kind, source)
        .await
        .unwrap();
    sqlx::query("INSERT INTO spaces_speech_projects(id,space_id,source_node_id,project_type,title,created_at,updated_at) VALUES('project','space','source',?,'Narration',1,1)")
        .bind(project_type)
        .execute(&mut *tx).await.unwrap();
    pending_output::reserve_in_transaction(
        &mut tx,
        "space",
        vec![PendingOutput {
            node_id: "run-output".into(),
            kind: output_kind,
            data: json!({
                "kind":output_kind,"title":"Narration","fileName":"Narration.wav",
                "assetPath":"/managed/reserved.wav","processingState":"processing"
            }),
            placement: pending_output::OutputPlacement::Derived(Size {
                width: 288.0,
                height: 80.0,
            }),
        }],
        &["source".to_owned()],
        "Speech narrated from source text",
        "run",
        None,
    )
    .await
    .unwrap();
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
async fn publication_settles_the_reserved_node_and_retries_are_idempotent() {
    let pool = fixture(
        "short_dubbing",
        nodes::tests::source_note("Source"),
        "audio",
    )
    .await;
    for _ in 0..2 {
        let mut tx = db::begin_write(&pool).await.unwrap();
        super::settle(&mut tx, "project", "run", &output(), None, None)
            .await
            .unwrap();
        tx.commit().await.unwrap();
    }
    let (kind, data): (String, String) =
        sqlx::query_as("SELECT kind,data FROM spaces_nodes WHERE id='run-output'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(kind, "audio");
    let data: serde_json::Value = serde_json::from_str(&data).unwrap();
    assert_eq!(data["assetPath"], output().path);
    assert_eq!(data["fileName"], "narration.wav");
    assert_eq!(data["durationSeconds"].as_u64(), Some(2));
    // The lineage was recorded once, when the Run reserved its output.
    let relations: Vec<(String, String, String)> =
        sqlx::query_as("SELECT source_id,target_id,kind FROM spaces_relations")
            .fetch_all(&pool)
            .await
            .unwrap();
    assert_eq!(
        relations,
        [(
            "run-output".to_owned(),
            "source".to_owned(),
            "derives-from".to_owned()
        )]
    );
    let mut changed = output();
    changed.sha256 = "b".repeat(64);
    let mut tx = db::begin_write(&pool).await.unwrap();
    assert!(
        super::settle(&mut tx, "project", "run", &changed, None, None)
            .await
            .is_err()
    );
}

#[tokio::test]
async fn a_dubbed_video_gets_its_own_cover_and_keeps_only_what_the_new_voice_leaves_true() {
    let mut source = json!({"kind":"video","title":"Interview","tags":["interview"],
        "assetPath":"/managed/source.mp4","fileName":"source.mp4","sourceRevision":"asset:1",
        "previewPath":"/previews/source.png","durationSeconds":2,
        "origin":{"kind":"web","url":"https://example.com/v","provider":"example","externalId":"v"},
        "mediaAnalysis":{"schemaVersion":1,"sourceRevision":"asset:1","durationMs":2_000,"waveform":[0.5]}});
    source["visualKnowledge"] = json!({
        "sourceRevision":visual_revision(&source).unwrap().unwrap(),"description":"Two people talking",
        "tags":[],"generatedAt":1,"executionSource":{"type":"model","provider":"openai","model":"vision"}
    });
    let pool = fixture("video_dubbing", source, "video").await;
    let cover = GeneratedPreview {
        path: "/previews/dubbed.png".into(),
        width: 1280,
        height: 720,
    };
    let mut dubbed = output();
    dubbed.path = "/managed/dubbed.mp4".into();
    let mut tx = db::begin_write(&pool).await.unwrap();
    super::settle(&mut tx, "project", "run", &dubbed, Some(cover), None)
        .await
        .unwrap();
    tx.commit().await.unwrap();
    let data: String = sqlx::query_scalar("SELECT data FROM spaces_nodes WHERE id='run-output'")
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
        "SELECT job_type FROM spaces_enrichment_jobs WHERE node_id='run-output'",
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(queued, ["analyze_media"]);
}

#[tokio::test]
async fn publication_is_atomic_and_a_deleted_source_cannot_receive_outputs() {
    let pool = fixture(
        "short_dubbing",
        nodes::tests::source_note("Source"),
        "audio",
    )
    .await;
    let mut tx = db::begin_write(&pool).await.unwrap();
    super::settle(&mut tx, "project", "run", &output(), None, None)
        .await
        .unwrap();
    tx.rollback().await.unwrap();
    let settled: Option<String> = sqlx::query_scalar(
        "SELECT json_extract(data,'$.processingState') FROM spaces_nodes WHERE id='run-output'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(settled.as_deref(), Some("processing"));
    sqlx::query("UPDATE spaces_nodes SET deleted_at=2 WHERE id='source'")
        .execute(&pool)
        .await
        .unwrap();
    let mut tx = db::begin_write(&pool).await.unwrap();
    assert!(
        super::settle(&mut tx, "project", "run", &output(), None, None)
            .await
            .is_err()
    );
}

#[tokio::test]
async fn an_output_whose_run_ended_without_publishing_stops_waiting() {
    let pool = fixture(
        "short_dubbing",
        nodes::tests::source_note("Source"),
        "audio",
    )
    .await;
    sqlx::query("UPDATE spaces_runs SET state='failed',completed_at=2 WHERE id='run'")
        .execute(&pool)
        .await
        .unwrap();
    pending_output::settle_abandoned(&pool).await.unwrap();
    let state: Option<String> = sqlx::query_scalar(
        "SELECT json_extract(data,'$.processingState') FROM spaces_nodes WHERE id='run-output'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(state.as_deref(), Some("failed"));
    let waiting: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM spaces_reserved_outputs")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(waiting, 0);
}
