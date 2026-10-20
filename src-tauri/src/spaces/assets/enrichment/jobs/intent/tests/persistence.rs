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
    synchronize_for_node(&mut tx, "space", "node", data, 2, None)
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

#[tokio::test]
async fn execution_choice_is_frozen_at_creation_and_survives_retry_and_restart() {
    use crate::ai::{contracts::ExternalAgentKind, tasks::TaskTargetSelection};
    let data = json!({"kind":"image","title":"Image","sourceRevision":"asset-1"});
    let pool = fixture(&data).await;
    let first = TaskTargetSelection::Agent {
        runtime_kind: ExternalAgentKind::Codex,
    };
    let next = TaskTargetSelection::Agent {
        runtime_kind: ExternalAgentKind::QoderCli,
    };
    let mut tx = pool.begin().await.unwrap();
    synchronize_for_node(&mut tx, "space", "node", &data, 2, Some(&first))
        .await
        .unwrap();
    tx.commit().await.unwrap();
    let jobs = persistence::list_actionable(&pool, None).await.unwrap();
    let id = &jobs[0].id;
    assert_eq!(jobs[0].execution_target, Some(first.clone()));
    assert_eq!(
        persistence::freeze_target(&pool, id, &next).await.unwrap(),
        first
    );
    let mut tx = pool.begin().await.unwrap();
    synchronize_for_node(&mut tx, "space", "node", &data, 3, Some(&next))
        .await
        .unwrap();
    tx.commit().await.unwrap();
    sqlx::query("UPDATE spaces_enrichment_jobs SET status='running' WHERE id=$1")
        .bind(id)
        .execute(&pool)
        .await
        .unwrap();
    persistence::reconcile_interrupted(&pool, 4).await.unwrap();
    let jobs = persistence::list_actionable(&pool, None).await.unwrap();
    assert_eq!(jobs[0].execution_target, Some(first.clone()));
    sqlx::query("UPDATE spaces_enrichment_jobs SET status='failed' WHERE id=$1")
        .bind(id)
        .execute(&pool)
        .await
        .unwrap();
    persistence::set_status(&pool, id, "'failed'", "pending", None, 5)
        .await
        .unwrap();
    assert_eq!(
        persistence::freeze_target(&pool, id, &next).await.unwrap(),
        first
    );
}

#[tokio::test]
async fn actionable_projection_identifies_each_live_job_owner_across_spaces() {
    let data = json!({"kind":"image","title":"First image","sourceRevision":"asset-1"});
    let pool = fixture(&data).await;
    sqlx::query("INSERT INTO spaces(id,kind,title,created_at,updated_at) VALUES('other','space','Other Space',1,1)")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO spaces_nodes(id,space_id,kind,initial_x,initial_y,data,created_at,updated_at) VALUES('other-node','other','image',0,0,$1,1,1)")
        .bind(json!({"kind":"image","title":"Second image","sourceRevision":"asset-2"}).to_string())
        .execute(&pool)
        .await
        .unwrap();

    let mut tx = pool.begin().await.unwrap();
    synchronize_for_node(&mut tx, "space", "node", &data, 2, None)
        .await
        .unwrap();
    synchronize_for_node(
        &mut tx,
        "other",
        "other-node",
        &json!({"kind":"image","title":"Second image","sourceRevision":"asset-2"}),
        3,
        None,
    )
    .await
    .unwrap();
    tx.commit().await.unwrap();

    let projections = persistence::list_actionable_projection(&pool, None)
        .await
        .unwrap();
    assert_eq!(projections.len(), 2);
    assert_eq!(projections[0].job.space_id, "space");
    assert_eq!(projections[0].space_title, "Space");
    assert_eq!(projections[0].node_title, "First image");
    assert_eq!(projections[0].node_kind, "image");
    assert_eq!(projections[1].job.space_id, "other");
    assert_eq!(projections[1].space_title, "Other Space");
    assert_eq!(projections[1].node_title, "Second image");

    let other_job_id = projections[1].job.id.clone();
    sqlx::query("UPDATE spaces SET title='Renamed Space' WHERE id='other'")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("UPDATE spaces_nodes SET data=json_set(data,'$.title','Renamed image') WHERE id='other-node'")
        .execute(&pool)
        .await
        .unwrap();
    let renamed = persistence::list_actionable_projection(&pool, None)
        .await
        .unwrap();
    assert_eq!(renamed[1].job.id, other_job_id);
    assert_eq!(renamed[1].space_title, "Renamed Space");
    assert_eq!(renamed[1].node_title, "Renamed image");

    sqlx::query("UPDATE spaces SET archived_at=4 WHERE id='other'")
        .execute(&pool)
        .await
        .unwrap();
    let projected = persistence::list_actionable_projection(&pool, None)
        .await
        .unwrap();
    assert_eq!(projected.len(), 1);
    let schedulable = persistence::list_actionable(&pool, None).await.unwrap();
    assert_eq!(schedulable.len(), 2);
    assert_eq!(schedulable[1].id, other_job_id);

    sqlx::query("UPDATE spaces SET archived_at=NULL WHERE id='other'")
        .execute(&pool)
        .await
        .unwrap();
    let restored = persistence::list_actionable_projection(&pool, None)
        .await
        .unwrap();
    assert_eq!(restored[1].job.id, other_job_id);

    sqlx::query("UPDATE spaces_nodes SET deleted_at=4 WHERE id='other-node'")
        .execute(&pool)
        .await
        .unwrap();
    let projections = persistence::list_actionable_projection(&pool, None)
        .await
        .unwrap();
    assert_eq!(projections.len(), 1);
    assert_eq!(projections[0].job.node_id, "node");
}
