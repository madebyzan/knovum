use std::{fs, io::Write, path::Path};

use serde_json::json;
use tokio_util::sync::CancellationToken;

use super::{
    commands::matches_existing_request,
    contracts::{SpeechSourceSeparationReviewStatus, StartSpeechSourceSeparationRequest},
    execution::candidate_matches_claim,
    media::{pcm_wav_metadata, MAX_SAMPLE_COUNT, SAMPLE_RATE},
    repository::{load_candidate, persist_candidate, transition_review},
    review_warning, MODEL_SPEC_ID, WORKFLOW,
};
use crate::{
    ai::local_runtime::separation::SeparatedAudioMetadata,
    spaces::{runs::AttemptClaim, speech::video_config::SpeechMediaSource},
};

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
        source: source(),
    };
    let input = json!({"workflow":WORKFLOW,"source":request.source.clone()}).to_string();
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
    write_sparse_wav(&exact, MAX_SAMPLE_COUNT);
    assert_eq!(pcm_wav_metadata(&exact).unwrap().1, MAX_SAMPLE_COUNT);

    let over = root.join("over.wav");
    write_sparse_wav(&over, MAX_SAMPLE_COUNT + 1);
    assert!(pcm_wav_metadata(&over)
        .unwrap_err()
        .contains("no more than ten minutes"));
    fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn candidate_commit_requires_a_live_attempt_and_review_is_compare_and_swap() {
    let pool = database().await;
    let run_id = uuid::Uuid::new_v4().to_string();
    let step_id = uuid::Uuid::new_v4().to_string();
    let attempt_id = uuid::Uuid::new_v4().to_string();
    let now = crate::clock::unix_time_millis().unwrap();
    sqlx::query("INSERT INTO spaces_runs(id,home_space_id,principal_kind,principal_id,kind,state,execution_epoch,policy_snapshot,input_snapshot,budget,created_at,updated_at) VALUES(?,'space','user','local-user','workflow','running',2,'{}','{}','{}',1,1)")
        .bind(&run_id).execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO spaces_run_steps(id,run_id,definition_node_id,instance_path,node_kind,state,resolved_input,created_at,updated_at) VALUES(? ,?,'separate','separate','speech_source_separation','running','{}',1,1)")
        .bind(&step_id).bind(&run_id).execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO spaces_run_attempts(id,run_id,step_instance_id,attempt_number,state,request_digest,executor_id,idempotency_key,execution_epoch,lease_until,started_at) VALUES(?,?,?,1,'executing',?,'executor','key',2,?,1)")
        .bind(&attempt_id).bind(&run_id).bind(&step_id).bind("b".repeat(64)).bind(now - 1)
        .execute(&pool).await.unwrap();
    let claim = AttemptClaim {
        attempt_id: attempt_id.clone(),
        run_id: run_id.clone(),
        step_instance_id: step_id.clone(),
        execution_epoch: 2,
        attempt_number: 1,
        home_space_id: "space".into(),
        node_kind: "speech_source_separation".into(),
        configuration: json!({}),
        resolved_input: json!({}),
        policy_snapshot: json!({}),
        budget: json!({}),
        cancellation: CancellationToken::new(),
    };
    let background = Path::new("/managed/background.wav");
    let metadata = SeparatedAudioMetadata {
        sha256: "c".repeat(64),
        sample_rate: SAMPLE_RATE,
        sample_count: SAMPLE_RATE.into(),
    };
    let error = persist_candidate(&pool, &claim, &source(), background, &metadata, 1_000)
        .await
        .unwrap_err();
    assert!(error.contains("attempt-stale"));

    sqlx::query("UPDATE spaces_run_attempts SET lease_until=? WHERE id=?")
        .bind(now + 60_000)
        .bind(&attempt_id)
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO spaces_speech_asset_publications(id,space_id,owner_kind,owner_id,asset_path,part_path,sha256,run_id,run_step_id,run_attempt_id,run_execution_epoch,created_at) VALUES('publication','space','source_separation',?,?,NULL,?,?,?,?,2,1)")
        .bind(&run_id).bind(background.to_string_lossy().as_ref()).bind(&metadata.sha256)
        .bind(&run_id).bind(&step_id).bind(&attempt_id).execute(&pool).await.unwrap();
    persist_candidate(&pool, &claim, &source(), background, &metadata, 1_000)
        .await
        .unwrap();

    let reviewed_at = crate::clock::unix_time_millis().unwrap() + 1;
    transition_review(
        &pool,
        &run_id,
        SpeechSourceSeparationReviewStatus::Pending,
        true,
        reviewed_at,
    )
    .await
    .unwrap();
    let candidate = load_candidate(&pool, &run_id).await.unwrap();
    assert!(candidate_matches_claim(&claim, &source(), &candidate));
    let mut changed_source = source();
    changed_source.source_revision = format!("sha256:{}", "d".repeat(64));
    assert!(!candidate_matches_claim(
        &claim,
        &changed_source,
        &candidate
    ));
    assert_eq!(
        candidate.review_status,
        SpeechSourceSeparationReviewStatus::Accepted
    );
    assert!(transition_review(
        &pool,
        &run_id,
        SpeechSourceSeparationReviewStatus::Pending,
        false,
        reviewed_at + 1,
    )
    .await
    .unwrap_err()
    .contains("concurrently"));
    assert_eq!(
        load_candidate(&pool, &run_id).await.unwrap().model_spec_id,
        MODEL_SPEC_ID
    );
    assert_eq!(
        load_candidate(&pool, &run_id).await.unwrap().review_warning,
        review_warning()
    );
}

fn write_sparse_wav(path: &Path, sample_count: u64) {
    let data_bytes = sample_count * 4;
    let mut header = [0_u8; 44];
    header[0..4].copy_from_slice(b"RIFF");
    header[4..8].copy_from_slice(&u32::try_from(data_bytes + 36).unwrap().to_le_bytes());
    header[8..12].copy_from_slice(b"WAVE");
    header[12..16].copy_from_slice(b"fmt ");
    header[16..20].copy_from_slice(&16_u32.to_le_bytes());
    header[20..22].copy_from_slice(&1_u16.to_le_bytes());
    header[22..24].copy_from_slice(&2_u16.to_le_bytes());
    header[24..28].copy_from_slice(&SAMPLE_RATE.to_le_bytes());
    header[28..32].copy_from_slice(&(SAMPLE_RATE * 4).to_le_bytes());
    header[32..34].copy_from_slice(&4_u16.to_le_bytes());
    header[34..36].copy_from_slice(&16_u16.to_le_bytes());
    header[36..40].copy_from_slice(b"data");
    header[40..44].copy_from_slice(&u32::try_from(data_bytes).unwrap().to_le_bytes());
    let mut file = fs::File::create(path).unwrap();
    file.write_all(&header).unwrap();
    file.set_len(data_bytes + 44).unwrap();
}
