use serde_json::json;
use sqlx::sqlite::SqlitePoolOptions;

use super::{validate_saved_provenance, SaveContext, NODE_KIND, WORKFLOW};
use crate::spaces::speech::{
    contracts::SpeechSegmentInput,
    video_config::{SpeechMediaSource, VideoDubbingOutputConfig, VideoTranslationProvenance},
};

#[tokio::test]
async fn accepts_user_edited_text_on_first_translation_save() {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    for statement in [
        "CREATE TABLE spaces_runs(id TEXT PRIMARY KEY,state TEXT,home_space_id TEXT,input_snapshot TEXT)",
        "CREATE TABLE spaces_speech_project_revisions(id TEXT PRIMARY KEY,project_id TEXT,revision INTEGER,output_config TEXT)",
        "CREATE TABLE spaces_speech_segments(id TEXT,project_revision_id TEXT,ordinal INTEGER,text TEXT,source_segment_id TEXT,source_segment_revision INTEGER,source_text TEXT)",
        "CREATE TABLE spaces_run_steps(id TEXT,run_id TEXT,node_kind TEXT,state TEXT,configuration TEXT,output TEXT)",
    ] {
        sqlx::query(statement).execute(&pool).await.unwrap();
    }
    let run_id = "11111111-1111-4111-8111-111111111111";
    let source = json!({"sourceSegmentId":"source","sourceSegmentRevision":1,"text":"Original"});
    let target =
        json!({"type":"model","connectionId":"connection","modelId":"model","temperature":0.0});
    let input = json!({
        "workflow":WORKFLOW,"projectId":"project","revisionId":"base","expectedRevision":1,
        "inputRevision":"sha256:input","targetLanguage":"zh-CN","localOnly":false,
        "provider":"openai","model":"model","baseUrl":"https://example.test/v1",
        "executionTarget":target,"segments":[source.clone()]
    });
    let configuration = json!({
        "projectId":"project","revisionId":"base","expectedRevision":1,
        "inputRevision":"sha256:input","targetLanguage":"zh-CN","localOnly":false,
        "provider":"openai","model":"model","baseUrl":"https://example.test/v1",
        "executionTarget":target,"ordinal":0,"segment":source
    });
    let output = json!({
        "stepId":"step","sourceSegmentId":"source","sourceSegmentRevision":1,
        "sourceText":"Original","translatedText":"Model translation"
    });
    sqlx::query("INSERT INTO spaces_runs VALUES(?, 'succeeded', 'space', ?)")
        .bind(run_id)
        .bind(input.to_string())
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO spaces_speech_project_revisions VALUES('base','project',1,'{}')")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO spaces_speech_segments VALUES('source','base',0,'Original',NULL,NULL,NULL)",
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query("INSERT INTO spaces_run_steps VALUES('step',?,?, 'succeeded',?,?)")
        .bind(run_id)
        .bind(NODE_KIND)
        .bind(configuration.to_string())
        .bind(output.to_string())
        .execute(&pool)
        .await
        .unwrap();
    let requested = VideoDubbingOutputConfig {
        format: "mp4".into(),
        video: SpeechMediaSource {
            asset_path: "video.mp4".into(),
            source_revision: format!("sha256:{}", "a".repeat(64)),
            duration_ms: 1_000,
        },
        background: None,
        background_separation: None,
        preserve_original_outside_dialogue: false,
        source_dialogue_intervals: Vec::new(),
        retain_original_track: false,
        voice_gain_db: 0.0,
        background_gain_db: 0.0,
        translation: Some(VideoTranslationProvenance {
            run_id: run_id.into(),
            target_language: "zh-CN".into(),
            provider: "openai".into(),
            model: "model".into(),
            local_only: false,
        }),
    };
    let segments = [SpeechSegmentInput {
        id: "edited".into(),
        chapter_key: None,
        speaker_key: None,
        text: "Human edited translation".into(),
        voice_revision_id: "voice".into(),
        language: "zh-CN".into(),
        target_start_ms: Some(0),
        target_end_ms: Some(1_000),
        source_segment_id: Some("source".into()),
        source_segment_revision: Some(1),
        source_text: Some("Original".into()),
    }];
    let active_output = json!({});
    let mut tx = pool.begin().await.unwrap();
    validate_saved_provenance(
        &mut tx,
        SaveContext {
            space_id: "space",
            project_id: "project",
            active_revision_id: "base",
            active_revision: 1,
            active_output_config: &active_output,
            requested_output_config: &requested,
            segments: &segments,
        },
    )
    .await
    .unwrap();
    for (pointer, replacement) in [
        ("/executionTarget/temperature", json!(0.5)),
        ("/executionTarget/modelId", json!("other-model")),
        ("/segment/sourceSegmentRevision", json!(2)),
        ("/segment/text", json!("Different source")),
    ] {
        let mut changed = configuration.clone();
        *changed.pointer_mut(pointer).unwrap() = replacement;
        sqlx::query("UPDATE spaces_run_steps SET configuration=? WHERE id='step'")
            .bind(changed.to_string())
            .execute(&mut *tx)
            .await
            .unwrap();
        assert!(validate_saved_provenance(
            &mut tx,
            SaveContext {
                space_id: "space",
                project_id: "project",
                active_revision_id: "base",
                active_revision: 1,
                active_output_config: &active_output,
                requested_output_config: &requested,
                segments: &segments,
            },
        )
        .await
        .is_err());
    }
}
