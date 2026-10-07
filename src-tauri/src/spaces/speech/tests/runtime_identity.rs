use super::*;
use serde_json::json;

#[test]
fn mixed_voice_runs_freeze_both_packages_and_reject_obsolete_identity() {
    let snapshot = json!({"runtimeDigests": {"light-inference": "a".repeat(64), "voice-generation": "b".repeat(64)}});
    let digests = read_snapshot(&snapshot).unwrap();
    assert_eq!(
        digests[&profile_for_engine("kokoro").unwrap()],
        "a".repeat(64)
    );
    assert_eq!(
        digests[&profile_for_engine("voxcpm2").unwrap()],
        "b".repeat(64)
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
    let project_id = uuid::Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO spaces_nodes(id,space_id,kind,initial_x,initial_y,data,created_at,updated_at) VALUES('source','space','note',0,0,'{\"kind\":\"note\",\"title\":\"Source\",\"plainText\":\"Hello\"}',1,1)")
        .execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO spaces_speech_projects(id,space_id,source_node_id,project_type,title,created_at,updated_at) VALUES(?,'space','source','short_dubbing','Draft',1,1)")
        .bind(&project_id).execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO spaces_speech_project_revisions(id,project_id,revision,source_revision,pronunciation_lexicon,output_config,created_at) VALUES('revision',?,1,'source','{}',?,1)")
        .bind(&project_id).bind(json!({"format":"wav","wavPrecision":"pcm16"}).to_string()).execute(&pool).await.unwrap();
    sqlx::query("UPDATE spaces_speech_projects SET active_revision_id='revision' WHERE id=?")
        .bind(&project_id)
        .execute(&pool)
        .await
        .unwrap();
    for (ordinal, engine) in [(0, "kokoro"), (1, "voxcpm2")] {
        sqlx::query("INSERT INTO spaces_voice_profiles(id,space_id,name,source_kind,created_at,updated_at) VALUES(?,'space',?,'reference',1,1)")
            .bind(engine).bind(engine).execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO spaces_voice_revisions(id,profile_id,revision,engine_id,model_spec_id,language,reference_asset_id,reference_source_revision,synthesis_config,authorization,created_at) VALUES(?,?,1,?,'model','en','reference','sha256:reference','{}','{}',1)")
            .bind(engine).bind(engine).bind(engine).execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO spaces_speech_segments(id,project_revision_id,ordinal,text,normalized_text,voice_revision_id,language,state,created_at,updated_at) VALUES(?,'revision',?,'Hello','Hello',?,'en','ready',1,1)")
            .bind(engine).bind(ordinal).bind(engine).execute(&pool).await.unwrap();
    }
    let request_id = uuid::Uuid::new_v4().to_string();
    let request = || GenerateSpeechProjectRequest {
        request_id: request_id.clone(),
        project_id: project_id.clone(),
        expected_revision: 1,
    };
    let mut digests = RuntimeDigests::from([(RuntimeProfile::LightInference, "a".repeat(64))]);
    assert!(run_start::start(&pool, request(), &digests).await.is_err());
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM spaces_runs")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 0);
    digests.insert(RuntimeProfile::VoiceGeneration, "b".repeat(64));
    run_start::start(&pool, request(), &digests).await.unwrap();
    run_start::start(&pool, request(), &digests).await.unwrap();
    let snapshot: String = sqlx::query_scalar("SELECT input_snapshot FROM spaces_runs WHERE id=?")
        .bind(&request_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(
        read_snapshot(&serde_json::from_str(&snapshot).unwrap()).unwrap(),
        digests
    );
    digests.insert(RuntimeProfile::VoiceGeneration, "c".repeat(64));
    assert!(run_start::start(&pool, request(), &digests).await.is_err());
}
