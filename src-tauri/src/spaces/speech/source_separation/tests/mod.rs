mod wav_fixture;

use super::{
    commands::matches_existing_request,
    contracts::StartSpeechSourceSeparationRequest,
    media::{pcm_wav_metadata, MAX_SAMPLE_COUNT},
    WORKFLOW,
};
use crate::spaces::speech::video_config::SpeechMediaSource;
use serde_json::json;
use std::fs;
mod publication;

async fn database() -> sqlx::SqlitePool {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    crate::spaces::db::migrate(&pool).await.unwrap();
    sqlx::query(
        "INSERT INTO spaces(id,kind,title,created_at,updated_at) \
         VALUES('space','space','Space',1,1)",
    )
    .execute(&pool)
    .await
    .unwrap();
    let mut tx = crate::spaces::db::begin_write(&pool).await.unwrap();
    crate::spaces::structure::repository::initialize_empty_in_transaction(&mut tx, "space")
        .await
        .unwrap();
    let node_content = json!({"kind":"video","title":"Interview","fileName":"source.mp4",
        "assetPath":source().asset_path,"sourceRevision":source().source_revision,
        "mediaAnalysis":{"schemaVersion":1,"sourceRevision":source().source_revision,"durationMs":1000,"waveform":[0.5]}});
    let request = serde_json::from_value(json!({"spaceId":"space","mutationId":"source-create",
        "targetParentId":null,"nodes":[{"id":"media","kind":"video",
        "initialX":0,"initialY":0,"width":320,"height":220,"sizingMode":"content","sourceChunks":[],"data":node_content}]})).unwrap();
    crate::spaces::nodes::tests::insert_many_in_transaction(&mut tx, request, None)
        .await
        .unwrap();
    tx.commit().await.unwrap();
    pool
}

fn source() -> SpeechMediaSource {
    SpeechMediaSource {
        asset_path: "/managed/source.mp4".into(),
        source_revision: format!("sha256:{}", "a".repeat(64)),
        duration_ms: 1_000,
    }
}

#[test]
fn existing_run_idempotence_requires_the_exact_workflow_space_and_source() {
    let request = StartSpeechSourceSeparationRequest {
        request_id: uuid::Uuid::new_v4().to_string(),
        space_id: "space".into(),
        source_node_id: "media".into(),
        vocals_label: "Vocals".into(),
        background_label: "Background".into(),
        source: source(),
    };
    let input =
        json!({"workflow":WORKFLOW,"sourceNodeId":request.source_node_id,"vocalsLabel":request.vocals_label,"backgroundLabel":request.background_label,"source":request.source.clone(),"model":"spleeter2Stems"})
            .to_string();
    assert!(matches_existing_request(
        "space", "workflow", &input, &request
    ));
    assert!(!matches_existing_request(
        "other", "workflow", &input, &request
    ));
    assert!(!matches_existing_request(
        "space", "action", &input, &request
    ));
    assert!(!matches_existing_request(
        "space",
        "workflow",
        &json!({"workflow":WORKFLOW,"source":source(),"unexpected":true}).to_string(),
        &request,
    ));
}

#[test]
fn wav_metadata_accepts_exact_limit_and_rejects_one_extra_frame() {
    let root = std::env::temp_dir().join(format!("knovum-separation-{}", uuid::Uuid::new_v4()));
    fs::create_dir(&root).unwrap();
    let exact = root.join("exact.wav");
    wav_fixture::write_sparse_wav(&exact, MAX_SAMPLE_COUNT);
    assert_eq!(pcm_wav_metadata(&exact).unwrap().1, MAX_SAMPLE_COUNT);

    let over = root.join("over.wav");
    wav_fixture::write_sparse_wav(&over, MAX_SAMPLE_COUNT + 1);
    assert!(pcm_wav_metadata(&over)
        .unwrap_err()
        .contains("no more than ten minutes"));
    fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn separation_accepts_owned_audio_and_video_but_rejects_other_spaces_and_stale_analysis() {
    let pool = database().await;
    for kind in ["audio", "video"] {
        let node_content = json!({"kind":kind,"title":"Media","assetPath":source().asset_path,"sourceRevision":source().source_revision,"mediaAnalysis":{"schemaVersion":1,"sourceRevision":source().source_revision,"durationMs":1000,"waveform":[0.5]}});
        sqlx::query("UPDATE spaces_nodes SET kind=?,data=? WHERE id='media'")
            .bind(kind)
            .bind(node_content.to_string())
            .execute(&pool)
            .await
            .unwrap();
        let mut tx = pool.begin().await.unwrap();
        super::media::require_owned_source(&mut tx, "space", "media", &source())
            .await
            .unwrap();
        assert!(
            super::media::require_owned_source(&mut tx, "other", "media", &source())
                .await
                .is_err()
        );
        assert!(
            super::media::require_owned_source(&mut tx, "space", "different-node", &source())
                .await
                .is_err()
        );
        let mut changed = source();
        changed.duration_ms = 1001;
        assert!(
            super::media::require_owned_source(&mut tx, "space", "media", &changed)
                .await
                .is_err()
        );
        tx.rollback().await.unwrap();
    }
}
