use super::super::contracts::VideoAsset;
use super::*;
use sqlx::sqlite::SqlitePoolOptions;

#[tokio::test]
async fn admits_only_ready_current_sources_in_the_owning_space() {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    sqlx::query("CREATE TABLE spaces_nodes(id TEXT PRIMARY KEY, space_id TEXT, kind TEXT, data TEXT, deleted_at INTEGER)").execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO spaces_nodes VALUES('source','space','video', $1, NULL)")
        .bind(r#"{"assetPath":"/managed/source.mp4","sourceRevision":"revision"}"#)
        .execute(&pool)
        .await
        .unwrap();
    let asset = VideoAsset {
        node_id: "source".into(),
        source_revision: "revision".into(),
        kind: VideoAssetKind::Video,
        duration_ms: 4000,
        width: 1280,
        height: 720,
    };
    let mut tx = pool.begin().await.unwrap();
    assert!(load(&mut tx, "space", &asset).await.is_ok());
    assert!(load(&mut tx, "foreign", &asset).await.is_err());
    let mut stale = asset.clone();
    stale.source_revision = "replaced".into();
    assert!(load(&mut tx, "space", &stale).await.is_err());
    let mut wrong_kind = asset.clone();
    wrong_kind.kind = VideoAssetKind::Audio;
    assert!(load(&mut tx, "space", &wrong_kind).await.is_err());
    sqlx::query("UPDATE spaces_nodes SET data=$1")
        .bind(r#"{"sourceRevision":"revision","processingState":"processing"}"#)
        .execute(&mut *tx)
        .await
        .unwrap();
    assert!(load(&mut tx, "space", &asset).await.is_err());
    sqlx::query("UPDATE spaces_nodes SET deleted_at=1")
        .execute(&mut *tx)
        .await
        .unwrap();
    assert!(load(&mut tx, "space", &asset).await.is_err());
}
