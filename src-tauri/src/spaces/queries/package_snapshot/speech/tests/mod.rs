use super::super::package_assets::PackageAssets;

#[tokio::test]
async fn snapshot_preserves_source_ownership_and_excludes_trashed_sources() {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    crate::spaces::db::migrate(&pool).await.unwrap();
    sqlx::query("INSERT INTO spaces(id,kind,title,created_at,updated_at) VALUES('space','space','Space',1,1)")
        .execute(&pool).await.unwrap();
    for source in ["live", "trash"] {
        sqlx::query("INSERT INTO spaces_nodes(id,space_id,kind,initial_x,initial_y,data,created_at,updated_at) VALUES(?,'space','note',0,0,'{\"kind\":\"note\",\"title\":\"Source\",\"plainText\":\"Hello\"}',1,1)")
            .bind(source).execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO spaces_speech_projects(id,space_id,source_node_id,project_type,title,created_at,updated_at) VALUES(?,'space',?,'short_dubbing','Narration',1,1)")
            .bind(source).bind(source).execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO spaces_speech_project_revisions(id,project_id,revision,source_revision,output_config,pronunciation_lexicon,created_at) VALUES(?,?,1,'source','{\"format\":\"wav\",\"wavPrecision\":\"pcm16\"}','{}',1)")
            .bind(format!("{source}-revision")).bind(source).execute(&pool).await.unwrap();
        sqlx::query("UPDATE spaces_speech_projects SET active_revision_id=? WHERE id=?")
            .bind(format!("{source}-revision"))
            .bind(source)
            .execute(&pool)
            .await
            .unwrap();
    }
    sqlx::query("UPDATE spaces_nodes SET deleted_at=2 WHERE id='trash'")
        .execute(&pool)
        .await
        .unwrap();
    let mut tx = pool.begin().await.unwrap();
    let snapshot = super::load(&mut tx, "space", &mut PackageAssets::default())
        .await
        .unwrap();
    assert!(snapshot.voice_revisions.is_empty());
    assert_eq!(snapshot.speech_projects.len(), 1);
    assert_eq!(snapshot.speech_projects[0]["sourceNodeId"], "live");
    assert_eq!(snapshot.project_revisions.len(), 1);
    assert_eq!(snapshot.project_revisions[0]["id"], "live-revision");
}
