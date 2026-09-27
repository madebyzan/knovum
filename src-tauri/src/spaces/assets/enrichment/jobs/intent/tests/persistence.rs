use serde_json::{json, Value};
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};

use super::super::synchronize_for_node;
use crate::spaces::assets::enrichment::jobs::{job::EnrichmentJobType, persistence, revision};

async fn fixture(data: &Value) -> SqlitePool {
    let pool = SqlitePoolOptions::new()
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

async fn synchronize(pool: &SqlitePool, data: &Value) {
    let mut tx = pool.begin().await.unwrap();
    sqlx::query("UPDATE spaces_nodes SET data=$1 WHERE id='node'")
        .bind(data.to_string())
        .execute(&mut *tx)
        .await
        .unwrap();
    synchronize_for_node(&mut tx, "space", "node", data, 2)
        .await
        .unwrap();
    tx.commit().await.unwrap();
}

#[tokio::test]
async fn images_and_covers_share_one_deduplicated_visual_job_type() {
    for data in [
        json!({"kind":"image","title":"Image","sourceRevision":"asset-1","imageEdit":null}),
        json!({"kind":"video","title":"Video","sourceRevision":"asset-1","previewPath":"/previews/cover.png"}),
    ] {
        let pool = fixture(&data).await;
        synchronize(&pool, &data).await;
        synchronize(&pool, &data).await;
        let jobs = persistence::list_actionable(&pool, Some("space"))
            .await
            .unwrap();
        let visual: Vec<_> = jobs
            .iter()
            .filter(|job| job.job_type == EnrichmentJobType::AnalyzeVisual)
            .collect();
        assert_eq!(visual.len(), 1);
        assert_eq!(
            visual[0].source_revision,
            revision::visual_revision(&data).unwrap().unwrap()
        );
    }
}

#[tokio::test]
async fn cover_replacement_cancels_running_visual_work_and_enqueues_the_new_input() {
    let mut data = json!({"kind":"video","title":"Video","sourceRevision":"asset-1","previewPath":"/previews/first.png"});
    let pool = fixture(&data).await;
    synchronize(&pool, &data).await;
    let first: String =
        sqlx::query_scalar("SELECT id FROM spaces_enrichment_jobs WHERE job_type='analyze_visual'")
            .fetch_one(&pool)
            .await
            .unwrap();
    sqlx::query("UPDATE spaces_enrichment_jobs SET status='running' WHERE id=$1")
        .bind(&first)
        .execute(&pool)
        .await
        .unwrap();
    data["previewPath"] = json!("/previews/second.png");
    synchronize(&pool, &data).await;
    let status: String =
        sqlx::query_scalar("SELECT status FROM spaces_enrichment_jobs WHERE id=$1")
            .bind(first)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(status, "cancelled");
    let jobs = persistence::list_actionable(&pool, Some("space"))
        .await
        .unwrap();
    assert_eq!(
        jobs.iter()
            .filter(|job| job.job_type == EnrichmentJobType::AnalyzeVisual)
            .count(),
        1
    );
    assert_eq!(
        jobs.iter()
            .filter(|job| job.job_type == EnrichmentJobType::AnalyzeMedia)
            .count(),
        1
    );
    data.as_object_mut().unwrap().remove("previewPath");
    synchronize(&pool, &data).await;
    assert!(persistence::list_actionable(&pool, Some("space"))
        .await
        .unwrap()
        .iter()
        .all(|job| job.job_type != EnrichmentJobType::AnalyzeVisual));
}

#[tokio::test]
async fn current_schema_rejects_both_obsolete_visual_job_names() {
    let data = json!({"kind":"image","title":"Image","sourceRevision":"asset-1"});
    let pool = fixture(&data).await;
    for old in ["analyze_image", "analyze_video_cover"] {
        assert!(sqlx::query("INSERT INTO spaces_enrichment_jobs(id,space_id,node_id,job_type,status,source_revision,created_at,updated_at) VALUES($1,'space','node',$1,'pending','asset-1',1,1)")
            .bind(old).execute(&pool).await.is_err());
    }
}
