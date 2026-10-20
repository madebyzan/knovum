mod cancellation;
mod configuration;
mod delivery_instruction;
mod ownership;
mod publication;
mod step_plan;

use crate::spaces::speech::asset_publication::PublicationOwnerKind;

use super::asset_publication::{self, PendingAsset};

pub(super) async fn database() -> sqlx::SqlitePool {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    crate::spaces::db::migrate(&pool).await.unwrap();
    sqlx::query("INSERT INTO spaces(id,kind,title,created_at,updated_at) VALUES('space','space','Space',1,1)")
        .execute(&pool).await.unwrap();
    pool
}

/// Builds a speech project with one segment per `segments` entry, each entry
/// naming the engine to synthesize with and the chapter it belongs to.
pub(super) async fn speech_project(
    pool: &sqlx::SqlitePool,
    project_type: &str,
    segments: &[(&str, Option<&str>)],
) -> String {
    let project_id = uuid::Uuid::new_v4().to_string();
    let mut tx = crate::spaces::db::begin_write(pool).await.unwrap();
    crate::spaces::structure::repository::initialize_empty_in_transaction(&mut tx, "space")
        .await
        .unwrap();
    crate::spaces::nodes::tests::insert_source_in_transaction(
        &mut tx,
        "space",
        "source",
        "note",
        crate::spaces::nodes::tests::source_note("Source"),
    )
    .await
    .unwrap();
    tx.commit().await.unwrap();
    sqlx::query("INSERT INTO spaces_speech_projects(id,space_id,source_node_id,project_type,title,created_at,updated_at) VALUES(?,'space','source',?,'Draft',1,1)")
        .bind(&project_id).bind(project_type).execute(pool).await.unwrap();
    let output_config = if project_type == "audiobook" {
        let mut chapters = Vec::new();
        for (_, chapter_key) in segments {
            if let Some(key) = chapter_key {
                if !chapters
                    .iter()
                    .any(|c: &serde_json::Value| c["key"] == *key)
                {
                    chapters.push(serde_json::json!({
                        "key": key,
                        "title": format!("Chapter {}", chapters.len() + 1)
                    }));
                }
            }
        }
        let mut roles = Vec::new();
        for (engine, _) in segments {
            if !roles
                .iter()
                .any(|r: &serde_json::Value| r["key"] == *engine)
            {
                let voice_id =
                    uuid::Uuid::new_v5(&uuid::Uuid::NAMESPACE_OID, engine.as_bytes()).to_string();
                roles.push(serde_json::json!({
                    "key": engine,
                    "voiceRevisionId": voice_id
                }));
            }
        }
        serde_json::json!({
            "format": "m4b",
            "sampleRate": 24000,
            "loudnessLufs": -20.0,
            "truePeakDb": -2.0,
            "paragraphPauseMs": 250,
            "chapterPauseMs": 1000,
            "wavPrecision": "pcm24",
            "metadata": {
                "title": "Audiobook",
                "author": "Author",
                "narrator": "Narrator",
                "cover": null
            },
            "chapters": chapters,
            "roles": roles
        })
    } else {
        serde_json::json!({"format":"wav","wavPrecision":"pcm16"})
    };
    sqlx::query("INSERT INTO spaces_speech_project_revisions(id,project_id,revision,source_revision,pronunciation_lexicon,output_config,created_at) VALUES('revision',?,1,'source','{}',?,1)")
        .bind(&project_id)
        .bind(output_config.to_string())
        .execute(pool).await.unwrap();
    sqlx::query("UPDATE spaces_speech_projects SET active_revision_id='revision' WHERE id=?")
        .bind(&project_id)
        .execute(pool)
        .await
        .unwrap();
    for (ordinal, (engine, chapter_key)) in segments.iter().enumerate() {
        let voice_id = if project_type == "audiobook" {
            uuid::Uuid::new_v5(&uuid::Uuid::NAMESPACE_OID, engine.as_bytes()).to_string()
        } else {
            (*engine).to_owned()
        };
        sqlx::query("INSERT OR IGNORE INTO spaces_voice_profiles(id,name,source_kind,created_at,updated_at) VALUES(?,?,'reference',1,1)")
            .bind(engine).bind(engine).execute(pool).await.unwrap();
        sqlx::query("INSERT OR IGNORE INTO spaces_voice_revisions(id,profile_id,revision,engine_id,model_spec_id,language,reference_asset_id,reference_source_revision,synthesis_config,authorization,created_at) VALUES(?,?,1,?,'model','en','reference','sha256:reference','{}','{}',1)")
            .bind(&voice_id).bind(engine).bind(engine).execute(pool).await.unwrap();
        let speaker_key = if project_type == "audiobook" {
            Some(*engine)
        } else {
            None
        };
        sqlx::query("INSERT INTO spaces_speech_segments(id,project_revision_id,ordinal,chapter_key,speaker_key,text,normalized_text,voice_revision_id,language,state,created_at,updated_at) VALUES(?,'revision',?,?,?,'Hello','Hello',?,'en','ready',1,1)")
            .bind(format!("segment-{ordinal}"))
            .bind(i64::try_from(ordinal).unwrap())
            .bind(*chapter_key)
            .bind(speaker_key)
            .bind(&voice_id)
            .execute(pool).await.unwrap();
    }
    project_id
}

pub(super) fn reservation_context() -> crate::spaces::speech::output_reservation::OutputContext {
    crate::spaces::speech::output_reservation::OutputContext::for_tests(
        std::env::temp_dir().join("knovum-speech-reservation"),
    )
}

#[tokio::test]
async fn voice_revision_cannot_change_under_existing_consumers() {
    let pool = database().await;
    sqlx::query("INSERT INTO spaces_voice_profiles(id,name,source_kind,created_at,updated_at) VALUES('voice','Voice','preset',1,1)")
        .execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO spaces_voice_revisions(id,profile_id,revision,engine_id,model_spec_id,language,preset_voice_id,synthesis_config,authorization,created_at) VALUES('revision','voice',1,'moss_nano','moss_nano-v1','en','6','{}','{}',1)")
        .execute(&pool).await.unwrap();
    let error =
        sqlx::query("UPDATE spaces_voice_revisions SET language='zh-CN' WHERE id='revision'")
            .execute(&pool)
            .await
            .unwrap_err();
    assert!(error.to_string().contains("VoiceRevision is immutable"));
    sqlx::query("UPDATE spaces_voice_profiles SET active_revision_id='revision' WHERE id='voice'")
        .execute(&pool)
        .await
        .unwrap();
    let language: String =
        sqlx::query_scalar("SELECT language FROM spaces_voice_revisions WHERE id='revision'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(language, "en");
}

#[tokio::test]
async fn failed_business_commit_keeps_durable_asset_cleanup_record() {
    let pool = database().await;
    let id = asset_publication::stage_unowned(
        &pool,
        &crate::spaces::authorization::RunScope::Application,
        PendingAsset {
            owner_kind: PublicationOwnerKind::VoiceReference,
            owner_id: "voice",
            asset_path: std::path::Path::new("/managed/voice.wav"),
            part_path: Some(std::path::Path::new("/managed/voice.wav.part")),
            sha256: &"a".repeat(64),
        },
    )
    .await
    .unwrap();
    let mut tx = pool.begin().await.unwrap();
    asset_publication::remove_in_transaction(&mut tx, &id)
        .await
        .unwrap();
    tx.rollback().await.unwrap();
    let paths: (String, String) = sqlx::query_as(
        "SELECT asset_path,part_path FROM spaces_speech_asset_publications WHERE id=?",
    )
    .bind(&id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(
        paths,
        (
            "/managed/voice.wav".into(),
            "/managed/voice.wav.part".into()
        )
    );
    let mut tx = pool.begin().await.unwrap();
    asset_publication::remove_in_transaction(&mut tx, &id)
        .await
        .unwrap();
    tx.commit().await.unwrap();
    let remaining: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM spaces_speech_asset_publications")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(remaining, 0);
}

#[test]
fn long_sentence_split_preserves_decimal_numbers_and_word_boundaries() {
    let sentence = format!("{} price 3.14 remains exact.", "recognizedword ".repeat(60));
    let chunks = super::text::segment(&sentence).unwrap();
    assert!(chunks.iter().any(|chunk| chunk.contains("3.14")));
    assert_eq!(chunks.join(" "), super::text::normalize(&sentence));
    assert!(chunks.iter().all(|chunk| chunk.chars().count() <= 400));
}

#[tokio::test]
async fn voice_catalog_retains_distinct_historical_and_archived_revisions() {
    let pool = database().await;
    sqlx::query("INSERT INTO spaces_voice_profiles(id,name,source_kind,created_at,updated_at) VALUES('profile','Voice','preset',1,1)").execute(&pool).await.unwrap();
    for (id, revision, seed) in [("first", 1, 17), ("second", 2, 29)] {
        sqlx::query("INSERT INTO spaces_voice_revisions(id,profile_id,revision,engine_id,model_spec_id,language,preset_voice_id,synthesis_config,authorization,created_at) VALUES(?,'profile',?,'moss_nano','moss-tts-nano-100m-onnx-bundled-v2','en','6',?,'{}',1)")
            .bind(id).bind(revision).bind(serde_json::json!({"speed":1,"sampleRate":48000,"seed":seed}).to_string()).execute(&pool).await.unwrap();
    }
    sqlx::query("UPDATE spaces_voice_profiles SET active_revision_id='second',archived_at=2 WHERE id='profile'").execute(&pool).await.unwrap();
    let catalog = super::repository::list_voices(&pool).await.unwrap();
    assert_eq!(catalog.len(), 2);
    assert!(catalog
        .iter()
        .all(|voice| voice.profile_id == "profile" && voice.archived_at == Some(2)));
    assert!(catalog
        .iter()
        .any(|voice| voice.revision_id == "first" && !voice.is_active));
    let old = super::execution::load_voice(&pool, "first").await.unwrap();
    assert_eq!(
        super::execution::parse_config(&old.synthesis_config)
            .unwrap()
            .seed,
        17
    );
}

#[test]
fn pinned_preset_catalog_rejects_unknown_and_noncanonical_speakers() {
    assert!(super::preset_voices::require("9", "en").is_ok());
    assert!(super::preset_voices::require("10", "en").is_err());
    assert!(super::preset_voices::require("0", "en").is_err());
    assert!(super::preset_voices::require("0", "zh-CN").is_ok());
    assert!(super::preset_voices::require("00", "en").is_err());
    assert!(super::preset_voices::require("0", "ja").is_err());
}

#[test]
fn synthesis_seed_affects_cache_and_is_required_in_current_config() {
    let mut segment = super::contracts::SpeechSegmentExecution {
        id: "segment".into(),
        normalized_text: "Hello".into(),
        delivery_instruction: None,
        voice_revision_id: "revision".into(),
        language: "en".into(),
    };
    let voice = super::contracts::VoiceExecution {
        revision_id: "revision".into(),
        source_kind: "preset".into(),
        engine_id: "moss_nano".into(),
        model_spec_id: "moss-tts-nano-100m-onnx-bundled-v2".into(),
        language: "en".into(),
        preset_voice_id: Some("6".into()),
        reference_asset_id: None,
        reference_source_revision: None,
        reference_text: None,
        design_description: None,
        synthesis_config: "".into(),
    };
    let first = super::contracts::VoiceSynthesisConfig {
        speed: 1.0,
        sample_rate: 48000,
        seed: 17,
    };
    let second = super::contracts::VoiceSynthesisConfig { seed: 29, ..first };
    assert_ne!(
        super::execution::input_digest(
            &segment,
            &voice,
            &first,
            "runtime",
            "moss-nano-cpu-f32-threads1-v1"
        )
        .unwrap(),
        super::execution::input_digest(
            &segment,
            &voice,
            &second,
            "runtime",
            "moss-nano-cpu-f32-threads1-v1"
        )
        .unwrap()
    );
    let plain =
        super::execution::input_digest(&segment, &voice, &first, "runtime", "profile").unwrap();
    segment.delivery_instruction = Some("calm".into());
    assert_ne!(
        plain,
        super::execution::input_digest(&segment, &voice, &first, "runtime", "profile").unwrap()
    );
    assert!(super::contracts::parse_synthesis_config(
        &serde_json::json!({"speed":1,"sampleRate":48000})
    )
    .is_err());
}
