use super::{list, load, save, set_enabled};
use crate::extensions::contracts::SkillRevision;
use serde_json::json;
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};

async fn pool() -> SqlitePool {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    sqlx::migrate!("migrations/extensions")
        .run(&pool)
        .await
        .unwrap();
    pool
}

fn skill(plugin_id: Option<&str>, enabled: bool) -> SkillRevision {
    SkillRevision {
        id: "skill".into(),
        plugin_id: plugin_id.map(str::to_owned),
        name: "evidence-brief".into(),
        description: "Compare sources".into(),
        digest: "revision".into(),
        enabled,
        requirements: vec![],
        included_tools: vec![],
        unsupported_requirements: vec![],
    }
}

async fn plugin(pool: &SqlitePool, enabled: bool) {
    let document = json!({
        "id": "owner", "enabled": enabled, "builtin": false, "publication": null,
        "source": { "type": "directory", "path": "/plugin" },
        "revision": {
            "digest": "package", "sourceRevision": "source", "components": [],
            "connectors": [], "fileCount": 1, "byteLength": 1,
            "manifest": {
                "$schema": "https://test/plugin.schema.json", "name": "Knowledge",
                "version": null, "description": null, "author": null, "homepage": null,
                "repository": null, "license": null, "keywords": null, "extensions": null,
            },
        },
    });
    sqlx::query("INSERT INTO plugins(id,document) VALUES('owner',$1) ON CONFLICT(id) DO UPDATE SET document=excluded.document")
        .bind(sqlx::types::Json(document)).execute(pool).await.unwrap();
}

#[tokio::test]
async fn plugin_is_the_only_enablement_authority_for_its_skills() {
    let pool = pool().await;
    save(&pool, &skill(Some("owner"), false)).await.unwrap();
    plugin(&pool, true).await;
    assert!(load(&pool, "skill").await.unwrap().enabled);
    assert!(list(&pool).await.unwrap()[0].enabled);
    plugin(&pool, false).await;
    assert!(!load(&pool, "skill").await.unwrap().enabled);
    assert!(!list(&pool).await.unwrap()[0].enabled);
    plugin(&pool, true).await;
    assert!(load(&pool, "skill").await.unwrap().enabled);
}

#[tokio::test]
async fn plugin_skills_reject_both_independent_enablement_commands() {
    let pool = pool().await;
    plugin(&pool, true).await;
    save(&pool, &skill(Some("owner"), true)).await.unwrap();
    for enabled in [true, false] {
        assert_eq!(
            set_enabled(&pool, "skill", enabled).await.unwrap_err(),
            "plugin-skill-owned-by-plugin"
        );
    }
    assert!(load(&pool, "skill").await.unwrap().enabled);
}

#[tokio::test]
async fn personal_skills_keep_independent_enablement() {
    let pool = pool().await;
    save(&pool, &skill(None, true)).await.unwrap();
    set_enabled(&pool, "skill", false).await.unwrap();
    assert!(!load(&pool, "skill").await.unwrap().enabled);
    set_enabled(&pool, "skill", true).await.unwrap();
    assert!(list(&pool).await.unwrap()[0].enabled);
}

#[tokio::test]
async fn missing_plugin_ownership_is_an_error() {
    let pool = pool().await;
    save(&pool, &skill(Some("missing"), true)).await.unwrap();
    assert_eq!(
        load(&pool, "skill").await.unwrap_err(),
        "skill-owner-not-found"
    );
    assert_eq!(list(&pool).await.unwrap_err(), "skill-owner-not-found");
}
