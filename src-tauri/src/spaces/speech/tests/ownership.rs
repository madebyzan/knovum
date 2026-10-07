use super::database;

async fn node(pool: &sqlx::SqlitePool, id: &str, space: &str, kind: &str) {
    sqlx::query("INSERT INTO spaces_nodes(id,space_id,kind,initial_x,initial_y,data,created_at,updated_at) VALUES(?,?,?,0,0,?,1,1)")
        .bind(id).bind(space).bind(kind)
        .bind(serde_json::json!({"kind":kind,"title":id,"plainText":"Hello"}).to_string())
        .execute(pool).await.unwrap();
}

async fn project(
    pool: &sqlx::SqlitePool,
    id: &str,
    source: &str,
    kind: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO spaces_speech_projects(id,space_id,source_node_id,project_type,title,created_at,updated_at) VALUES(?,'space',?,?,?,1,1)")
        .bind(id).bind(source).bind(kind).bind(id).execute(pool).await?;
    Ok(())
}

#[tokio::test]
async fn projects_require_a_matching_source_node_in_the_same_space() {
    let pool = database().await;
    sqlx::query("INSERT INTO spaces(id,kind,title,created_at,updated_at) VALUES('other','space','Other',1,1)")
        .execute(&pool).await.unwrap();
    node(&pool, "text", "space", "note").await;
    node(&pool, "video", "space", "video").await;
    node(&pool, "foreign", "other", "note").await;
    assert!(project(&pool, "read", "text", "short_dubbing")
        .await
        .is_ok());
    assert!(project(&pool, "book", "text", "audiobook").await.is_ok());
    assert!(project(&pool, "dub", "video", "video_dubbing")
        .await
        .is_ok());
    for (id, source, kind) in [
        ("missing", "absent", "short_dubbing"),
        ("foreign", "foreign", "short_dubbing"),
        ("wrong-kind", "video", "short_dubbing"),
    ] {
        assert!(project(&pool, id, source, kind).await.is_err());
    }
    assert!(sqlx::query(
        "UPDATE spaces_speech_projects SET source_node_id='video' WHERE id='read'"
    )
    .execute(&pool)
    .await
    .is_err());
    assert!(
        sqlx::query("UPDATE spaces_nodes SET space_id='other' WHERE id='text'")
            .execute(&pool)
            .await
            .is_err()
    );
    assert!(
        sqlx::query("UPDATE spaces_nodes SET kind='audio' WHERE id='text'")
            .execute(&pool)
            .await
            .is_err()
    );
}

#[tokio::test]
async fn deleting_source_node_removes_its_projects() {
    let pool = database().await;
    node(&pool, "text", "space", "note").await;
    project(&pool, "read", "text", "short_dubbing")
        .await
        .unwrap();
    sqlx::query("DELETE FROM spaces_nodes WHERE id='text'")
        .execute(&pool)
        .await
        .unwrap();
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM spaces_speech_projects")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 0);
}

#[tokio::test]
async fn video_production_requires_the_current_node_asset_and_revision() {
    let pool = database().await;
    node(&pool, "video", "space", "video").await;
    sqlx::query("UPDATE spaces_nodes SET data=? WHERE id='video'")
        .bind(
            serde_json::json!({"kind":"video","title":"Video","assetPath":"/source.mp4",
            "sourceRevision":format!("sha256:{}", "a".repeat(64))})
            .to_string(),
        )
        .execute(&pool)
        .await
        .unwrap();
    let mut config = super::super::video_config::parse(&serde_json::json!({
        "format":"mp4","video":{"assetPath":"/source.mp4",
            "sourceRevision":format!("sha256:{}", "a".repeat(64)),"durationMs":1000},
        "background":null,"backgroundSeparation":null,"translation":null,
        "voiceGainDb":0,"backgroundGainDb":-12,"preserveOriginalOutsideDialogue":false,
        "sourceDialogueIntervals":[],"retainOriginalTrack":false
    }))
    .unwrap();
    let mut tx = pool.begin().await.unwrap();
    let current = super::super::project_config::ProjectOutputConfig::VideoDubbing(config.clone());
    super::super::project_source::require(&mut tx, "space", "video", &current)
        .await
        .unwrap();
    config.video.asset_path = "/another.mp4".into();
    let wrong = super::super::project_config::ProjectOutputConfig::VideoDubbing(config.clone());
    assert!(
        super::super::project_source::require(&mut tx, "space", "video", &wrong)
            .await
            .is_err()
    );
    config.video.asset_path = "/source.mp4".into();
    config.video.source_revision = format!("sha256:{}", "b".repeat(64));
    let wrong = super::super::project_config::ProjectOutputConfig::VideoDubbing(config);
    assert!(
        super::super::project_source::require(&mut tx, "space", "video", &wrong)
            .await
            .is_err()
    );
}
