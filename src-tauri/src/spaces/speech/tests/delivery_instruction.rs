use crate::spaces::speech::delivery_instruction;

#[test]
fn delivery_is_available_only_for_instruction_capable_reference_voices() {
    assert!(delivery_instruction::supports("voxcpm2", "reference", None));
    for (engine, source, transcript) in [
        ("moss_nano", "reference", None),
        ("voxcpm2", "designed", Some("Sample")),
        ("voxcpm2", "reference", Some("Sample")),
        ("moss_nano", "preset", None),
    ] {
        assert!(!delivery_instruction::supports(engine, source, transcript));
        assert!(delivery_instruction::validate(
            Some("calm"),
            delivery_instruction::supports(engine, source, transcript)
        )
        .is_err());
        assert!(delivery_instruction::validate(
            None,
            delivery_instruction::supports(engine, source, transcript)
        )
        .is_ok());
    }
}

#[test]
fn delivery_has_a_bounded_nonempty_canonical_input() {
    assert_eq!(
        delivery_instruction::validate(Some("  calm  "), true).unwrap(),
        Some("calm")
    );
    for input in [" ".to_owned(), "语".repeat(501)] {
        assert!(delivery_instruction::validate(Some(&input), true).is_err());
    }
    assert!(delivery_instruction::validate(Some(&"语".repeat(500)), true).is_ok());
}

#[tokio::test]
async fn voice_catalog_exposes_delivery_support_from_the_frozen_source() {
    let pool = super::database().await;
    for (id, source, transcript) in [
        ("guided", "reference", None),
        ("exact", "reference", Some("Selected sample")),
        ("design", "designed", Some("Selected sample")),
    ] {
        sqlx::query("INSERT INTO spaces_voice_profiles(id,space_id,name,source_kind,created_at,updated_at) VALUES(?,'space',?,?,1,1)")
            .bind(id).bind(id).bind(source).execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO spaces_voice_revisions(id,profile_id,revision,engine_id,model_spec_id,language,reference_asset_id,reference_source_revision,reference_text,design_description,synthesis_config,authorization,created_at) VALUES(?,?,1,'voxcpm2','voxcpm2-2b-gguf-orig-v1','en','/managed/sample.wav',?,?,?,'{}','{}',1)")
            .bind(id).bind(id).bind(format!("sha256:{}", "a".repeat(64))).bind(transcript)
            .bind(if source == "designed" { Some("Calm") } else { None })
            .execute(&pool).await.unwrap();
        sqlx::query("UPDATE spaces_voice_profiles SET active_revision_id=? WHERE id=?")
            .bind(id)
            .bind(id)
            .execute(&pool)
            .await
            .unwrap();
    }
    let catalog = crate::spaces::speech::repository::list_voices(&pool, "space")
        .await
        .unwrap();
    assert_eq!(catalog.len(), 3);
    for voice in catalog {
        assert_eq!(
            voice.supports_delivery_instruction,
            voice.revision_id == "guided"
        );
    }
}
