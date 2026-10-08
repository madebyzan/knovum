use super::{asset_publication, database, PendingAsset, PublicationOwnerKind};

#[tokio::test]
async fn seals_only_an_active_unowned_preparation_reservation() {
    let pool = database().await;
    let id = asset_publication::stage_unowned(
        &pool,
        "space",
        PendingAsset {
            owner_kind: PublicationOwnerKind::VoiceCandidateInput,
            owner_id: "candidate",
            asset_path: std::path::Path::new("/managed/candidate.input"),
            part_path: Some(std::path::Path::new("/managed/candidate.input.part")),
            sha256: &"0".repeat(64),
        },
    )
    .await
    .unwrap();
    let digest = "a".repeat(64);
    asset_publication::seal_unowned(&pool, &id, &digest)
        .await
        .unwrap();
    assert!(asset_publication::seal_unowned(&pool, &id, &"b".repeat(64))
        .await
        .is_err());
    assert!(asset_publication::seal_unowned(&pool, "missing", &digest)
        .await
        .is_err());
    let stored: String =
        sqlx::query_scalar("SELECT sha256 FROM spaces_speech_asset_publications WHERE id=?")
            .bind(&id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(stored, digest);
}
