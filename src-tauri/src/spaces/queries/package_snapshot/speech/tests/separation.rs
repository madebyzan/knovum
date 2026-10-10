use super::PackageAssets;
use serde_json::json;

#[tokio::test]
async fn separation_snapshot_preserves_source_and_both_assets_without_review_state() {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    crate::spaces::db::migrate(&pool).await.unwrap();
    sqlx::query("INSERT INTO spaces(id,kind,title,created_at,updated_at) VALUES('space','space','Space',1,1)")
        .execute(&pool).await.unwrap();
    for id in ["live", "trash"] {
        let node_content = json!({"kind":"audio","title":id,"fileName":"source.wav",
            "assetPath":format!("/{id}/source.wav"),"sourceRevision":format!("sha256:{}", "a".repeat(64))});
        sqlx::query("INSERT INTO spaces_nodes(id,space_id,kind,initial_x,initial_y,data,created_at,updated_at) VALUES(?,'space','audio',0,0,?,1,1)")
            .bind(id).bind(node_content.to_string()).execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO spaces_runs(id,scope_kind,home_space_id,principal_kind,principal_id,kind,state,policy_snapshot,input_snapshot,budget,created_at,updated_at,completed_at) VALUES(?,'space','space','user','local-user','workflow','succeeded','{}','{}','{}',1,1,1)")
            .bind(id).execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO spaces_speech_source_separations(id,space_id,run_id,source_node_id,source_asset_path,source_revision,source_duration_ms,model_spec_id,background_asset_path,background_sha256,vocals_asset_path,vocals_sha256,background_sample_rate,background_sample_count,background_duration_ms,created_at) VALUES(?,'space',?,?,? ,?,1000,'spleeter-2stems-fp16-cpu-v1',?,?,?, ?,44100,44100,1000,1)")
            .bind(id).bind(id).bind(id).bind(format!("/{id}/source.wav"))
            .bind(format!("sha256:{}", "a".repeat(64)))
            .bind(format!("/{id}/background.wav")).bind("b".repeat(64))
            .bind(format!("/{id}/vocals.wav")).bind("c".repeat(64))
            .execute(&pool).await.unwrap();
    }
    sqlx::query("UPDATE spaces_nodes SET deleted_at=2 WHERE id='trash'")
        .execute(&pool)
        .await
        .unwrap();
    let mut tx = pool.begin().await.unwrap();
    let mut assets = PackageAssets::default();
    let snapshot = super::super::load(&mut tx, "space", &mut assets)
        .await
        .unwrap();
    assert_eq!(snapshot.source_separations.len(), 1);
    let separation = &snapshot.source_separations[0];
    assert_eq!(separation["sourceNodeId"], "live");
    assert_eq!(separation["id"], separation["runId"]);
    assert!(separation.get("reviewStatus").is_none());
    let assets = assets.into_assets();
    assert_eq!(assets.len(), 3);
    for (field, path) in [
        ("sourceAssetPath", "/live/source.wav"),
        ("backgroundAssetPath", "/live/background.wav"),
        ("vocalsAssetPath", "/live/vocals.wav"),
    ] {
        let token = separation[field]
            .as_str()
            .unwrap()
            .strip_prefix(crate::spaces::assets::PACKAGE_ASSET_PREFIX)
            .unwrap();
        assert!(assets
            .iter()
            .any(|asset| asset.token == token && asset.path == path));
    }
}
