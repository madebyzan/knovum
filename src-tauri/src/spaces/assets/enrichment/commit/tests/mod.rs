use super::{publish_in_transaction, PublicationGuard};
use crate::app_error::AppError;
use serde_json::{json, Value};
use sqlx::SqlitePool;
use tokio_util::sync::CancellationToken;

pub(in crate::spaces::assets::enrichment) async fn fixture(data: &Value) -> SqlitePool {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    crate::spaces::db::migrate(&pool).await.unwrap();
    sqlx::query("INSERT INTO spaces(id,kind,title,created_at,updated_at) VALUES('space','space','Space',1,1)").execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO spaces_nodes(id,space_id,kind,initial_x,initial_y,data,created_at,updated_at) VALUES('node','space',$1,0,0,$2,1,1)")
        .bind(data["kind"].as_str().unwrap()).bind(data.to_string()).execute(&pool).await.unwrap();
    pool
}

pub(in crate::spaces::assets::enrichment) async fn publish(
    pool: &SqlitePool,
    data: &Value,
    fields: serde_json::Map<String, Value>,
    token: &CancellationToken,
) -> Result<Value, AppError> {
    let mut tx = crate::spaces::db::begin_write(pool).await?;
    let publication = publish_in_transaction(
        &mut tx,
        ("space", "node"),
        PublicationGuard {
            source_revision: data["sourceRevision"].as_str(),
            job_id: None,
            media_knowledge: data.get("mediaKnowledge"),
            document_coverage: data.get("documentCoverage"),
            cancellation: Some(token),
        },
        fields,
        None,
        None,
    )
    .await?;
    tx.commit().await.map_err(crate::spaces::db::sql_error)?;
    Ok(publication)
}

#[tokio::test]
async fn stale_sources_manual_edits_and_cancelled_results_cannot_publish() {
    let original = json!({"kind":"audio","title":"Record","fileName":"record.wav","assetPath":"/assets/record.wav","sourceRevision":"source-1","mediaKnowledge":{"original":true}});
    for change in ["source", "manual", "cancel"] {
        let pool = fixture(&original).await;
        let mut current = original.clone();
        let token = CancellationToken::new();
        match change {
            "source" => current["sourceRevision"] = json!("source-2"),
            "manual" => current["mediaKnowledge"] = json!({"manualEdit":true}),
            _ => token.cancel(),
        }
        sqlx::query("UPDATE spaces_nodes SET data=$1 WHERE id='node'")
            .bind(current.to_string())
            .execute(&pool)
            .await
            .unwrap();
        let error = publish(
            &pool,
            &original,
            serde_json::Map::from_iter([("title".into(), json!("Late result"))]),
            &token,
        )
        .await
        .unwrap_err();
        let expected = match change {
            "source" => "enrichment_source_obsolete",
            "manual" => "operation_failed",
            _ => "enrichment_cancelled",
        };
        assert!(error.code == expected, "{error}");
        let raw: String = sqlx::query_scalar("SELECT data FROM spaces_nodes WHERE id='node'")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(serde_json::from_str::<Value>(&raw).unwrap(), current);
    }
}

#[tokio::test]
async fn manual_document_coverage_edit_invalidates_an_older_analysis() {
    let original = json!({"kind":"file","title":"Document","sourceRevision":"source","documentCoverage":{"original":true}});
    let pool = fixture(&original).await;
    let mut changed = original.clone();
    changed["documentCoverage"] = json!({"manual":true});
    sqlx::query("UPDATE spaces_nodes SET data=$1 WHERE id='node'")
        .bind(changed.to_string())
        .execute(&pool)
        .await
        .unwrap();
    let error = publish(
        &pool,
        &original,
        serde_json::Map::new(),
        &CancellationToken::new(),
    )
    .await
    .unwrap_err();
    assert!(error.code == "document_coverage_obsolete");
}
