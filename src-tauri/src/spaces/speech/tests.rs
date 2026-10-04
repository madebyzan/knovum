use super::asset_publication::{self, PendingAsset};

async fn database() -> sqlx::SqlitePool {
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

#[tokio::test]
async fn voice_revision_cannot_change_under_existing_consumers() {
    let pool = database().await;
    sqlx::query("INSERT INTO spaces_voice_profiles(id,space_id,name,source_kind,created_at,updated_at) VALUES('voice','space','Voice','preset',1,1)")
        .execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO spaces_voice_revisions(id,profile_id,revision,engine_id,model_spec_id,language,preset_voice_id,synthesis_config,authorization,created_at) VALUES('revision','voice',1,'kokoro','kokoro-v1','en','0','{}','{}',1)")
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
        "space",
        PendingAsset {
            owner_kind: "voice_reference",
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
