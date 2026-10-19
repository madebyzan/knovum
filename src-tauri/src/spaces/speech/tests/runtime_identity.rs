use super::*;
use serde_json::json;

#[test]
fn mixed_voice_runs_freeze_one_inference_package_and_reject_obsolete_identity() {
    let snapshot = json!({"runtimeDigests": {"native-inference-v2": "a".repeat(64)}});
    let digests = read_snapshot(&snapshot).unwrap();
    assert_eq!(
        digests[&profile_for_engine("moss_nano").unwrap()],
        "a".repeat(64)
    );
    assert_eq!(
        digests[&profile_for_engine("voxcpm2").unwrap()],
        "a".repeat(64)
    );
    for invalid in [
        json!({"runtimeDigest": "a".repeat(64)}),
        json!({"runtimeDigests": {}}),
        json!({"runtimeDigests": {"agent": "a".repeat(64)}}),
        json!({"runtimeDigests": {"voice-generation": "invalid"}}),
    ] {
        assert!(read_snapshot(&invalid).is_err());
    }
}

#[tokio::test]
async fn project_generation_requires_every_engine_package_and_freezes_the_exact_map() {
    use crate::spaces::speech::{contracts::GenerateSpeechProjectRequest, run_start};
    let pool = crate::spaces::speech::tests::database().await;
    let project_id = crate::spaces::speech::tests::speech_project(
        &pool,
        "short_dubbing",
        &[("moss_nano", None), ("voxcpm2", None)],
    )
    .await;
    let request_id = uuid::Uuid::new_v4().to_string();
    let request = || GenerateSpeechProjectRequest {
        request_id: request_id.clone(),
        project_id: project_id.clone(),
        expected_revision: 1,
    };
    let outputs = crate::spaces::speech::tests::reservation_context();
    let mut digests = RuntimeDigests::new();
    assert!(run_start::start(&pool, request(), &digests, &outputs)
        .await
        .is_err());
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM spaces_runs")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 0);
    digests.insert("native-inference-v2".into(), "b".repeat(64));
    run_start::start(&pool, request(), &digests, &outputs)
        .await
        .unwrap();
    run_start::start(&pool, request(), &digests, &outputs)
        .await
        .unwrap();
    let snapshot: String = sqlx::query_scalar("SELECT input_snapshot FROM spaces_runs WHERE id=?")
        .bind(&request_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(
        read_snapshot(&serde_json::from_str(&snapshot).unwrap()).unwrap(),
        digests
    );
    digests.insert("native-inference-v2".into(), "c".repeat(64));
    assert!(run_start::start(&pool, request(), &digests, &outputs)
        .await
        .is_err());
}
