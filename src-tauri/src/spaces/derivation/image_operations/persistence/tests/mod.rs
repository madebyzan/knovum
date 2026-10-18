use std::path::Path;

use serde_json::{json, Value};
use sqlx::SqlitePool;
use tokio_util::sync::CancellationToken;

use super::super::{
    contracts::{DeriveImageRequest, ImageExecutionTarget, ImageOperation},
    output::CanonicalImage,
};
use crate::{
    local_resources::model_catalog::LocalModelId,
    spaces::{
        assets::enrichment::jobs::revision::visual_revision, db,
        derivation::action_execution::ExecutionAuthority, knowledge::model::revision,
        nodes::contracts::InsertNodesRequest, structure::repository,
    },
};

async fn fixture() -> (SqlitePool, DeriveImageRequest, Value) {
    let pool = crate::spaces::derivation::tests::running_action_pool("job").await;
    let mut source = json!({"kind":"image","title":"Source","fileName":"source.png","assetPath":"/owned/source.png","sourceRevision":"asset:1"});
    revision::apply(&mut source).unwrap();
    let mut output = json!({"kind":"image","title":"Depth","fileName":"depth.png","assetPath":"/owned/pending.png","processingState":"processing"});
    revision::apply(&mut output).unwrap();
    let mut tx = db::begin_write(&pool).await.unwrap();
    repository::initialize_empty_in_transaction(&mut tx, "space")
        .await
        .unwrap();
    let nodes: InsertNodesRequest = serde_json::from_value(json!({
        "spaceId":"space","mutationId":"create","targetParentId":null,
        "nodes":[
            {"id":"source","kind":"image","initialX":0,"initialY":0,"width":300,"height":200,"sizingMode":"frame","data":source,"sourceChunks":[]},
            {"id":"output","kind":"image","initialX":400,"initialY":0,"width":300,"height":200,"sizingMode":"frame","data":output,"sourceChunks":[]}
        ]
    })).unwrap();
    crate::spaces::nodes::tests::insert_many_in_transaction(&mut tx, nodes, None)
        .await
        .unwrap();
    sqlx::query("INSERT INTO spaces_knowledge_derivations(id,space_id,output_node_id,action_kind,action_version,engine,status,created_at,updated_at) VALUES('job','space','output','generate-image-depth',1,'local:test','running',1,1)")
        .execute(&mut *tx).await.unwrap();
    tx.commit().await.unwrap();
    let model = LocalModelId::DepthAnythingV2Small;
    (
        pool,
        DeriveImageRequest {
            id: "job".into(),
            space_id: "space".into(),
            source_node_id: "source".into(),
            output_node_id: "output".into(),
            source_revision: "asset:1".into(),
            source_edit: Value::Null,
            operation: ImageOperation::GenerateDepth,
            execution: ImageExecutionTarget::Local {
                model,
                spec_id: model.image_spec().unwrap(),
            },
        },
        source,
    )
}

async fn complete(
    pool: &SqlitePool,
    request: &DeriveImageRequest,
    token: &CancellationToken,
) -> Result<
    crate::spaces::derivation::CompletedDerivationOutput,
    crate::foundation::app_error::AppError,
> {
    super::complete_persistence(
        pool,
        request,
        &CanonicalImage {
            png: vec![1, 2, 3],
            width: 2,
            height: 1,
            digest: "asset:depth".into(),
        },
        Path::new("/owned/depth.png"),
        &ExecutionAuthority {
            run_id: "job".into(),
            execution_epoch: 0,
        },
        token,
        None,
    )
    .await
}

#[tokio::test]
async fn depth_publication_preserves_source_and_creates_one_related_output() {
    let (pool, request, original) = fixture().await;
    let publication = complete(&pool, &request, &CancellationToken::new())
        .await
        .unwrap();
    assert_eq!(
        publication.data["imageDepth"]["encoding"],
        "relative-inverse-depth-u16-v1"
    );
    assert_eq!(
        publication.data["imageDepth"]["modelSpecId"],
        LocalModelId::DepthAnythingV2Small.image_spec().unwrap()
    );
    assert!(publication.data.get("processingState").is_none());
    assert!(publication.data.get("imageEdit").is_none());
    let stored: String = sqlx::query_scalar("SELECT data FROM spaces_nodes WHERE id='source'")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(serde_json::from_str::<Value>(&stored).unwrap(), original);
    assert!(complete(&pool, &request, &CancellationToken::new())
        .await
        .is_err());
    // Lineage belongs to the reservation, so settling content never writes a Relation.
    let relation_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM spaces_relations")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(relation_count, 0);
}

#[tokio::test]
async fn stale_cancelled_or_changed_sources_cannot_publish() {
    for change in [
        "UPDATE spaces_runs SET cancel_requested_at=2 WHERE id='job'",
        "UPDATE spaces_runs SET execution_epoch=1 WHERE id='job'",
        "UPDATE spaces_nodes SET data=json_set(data,'$.sourceRevision','asset:2') WHERE id='source'",
        "UPDATE spaces_nodes SET data=json_set(data,'$.imageEdit',json('{}')) WHERE id='source'",
        "UPDATE spaces_nodes SET deleted_at=2 WHERE id='source'",
    ] {
        let (pool, request, _) = fixture().await;
        sqlx::query(change).execute(&pool).await.unwrap();
        assert!(complete(&pool, &request, &CancellationToken::new()).await.is_err(), "{change}");
        assert_unpublished(&pool).await;
    }
}

#[tokio::test]
async fn cancellation_rolls_back_the_output_relation_and_completion_together() {
    let (pool, request, _) = fixture().await;
    let token = CancellationToken::new();
    token.cancel();
    assert!(complete(&pool, &request, &token).await.is_err());
    assert_unpublished(&pool).await;
}

#[tokio::test]
async fn selection_extraction_publishes_a_photo_without_depth_metadata() {
    let (pool, mut request, _) = fixture().await;
    sqlx::query("UPDATE spaces_knowledge_derivations SET action_kind='extract-image-selection' WHERE id='job'")
        .execute(&pool).await.unwrap();
    let model = LocalModelId::EfficientSamTi;
    request.operation = ImageOperation::ExtractSelection {
        selection: super::super::contracts::ImageSelection {
            points: vec![crate::ai::contracts::image_compute::ImageSelectionPoint {
                x: 0.5,
                y: 0.5,
                foreground: true,
            }],
            box_rect: None,
            strokes: vec![],
        },
    };
    request.execution = ImageExecutionTarget::Local {
        model,
        spec_id: model.image_spec().unwrap(),
    };
    let publication = complete(&pool, &request, &CancellationToken::new())
        .await
        .unwrap();
    assert!(publication.data.get("imageDepth").is_none());
    assert_eq!(publication.data["assetPath"], "/owned/depth.png");
}

#[tokio::test]
async fn an_upscaled_picture_keeps_its_labels_and_what_it_shows() {
    let (pool, mut request, mut source) = fixture().await;
    source["tags"] = json!(["poster"]);
    let shown = visual_revision(&source).unwrap().unwrap();
    source["visualKnowledge"] = json!({
        "sourceRevision":shown,"description":"A red poster","tags":[],"generatedAt":1,
        "executionSource":{"type":"model","provider":"openai","model":"vision"}
    });
    revision::apply(&mut source).unwrap();
    sqlx::query("UPDATE spaces_nodes SET data=$1 WHERE id='source'")
        .bind(source.to_string())
        .execute(&pool)
        .await
        .unwrap();
    let model = LocalModelId::RealEsrganGeneralX4v3;
    request.operation = ImageOperation::Upscale {
        target_long_edge: 2048,
    };
    request.execution = ImageExecutionTarget::Local {
        model,
        spec_id: model.image_spec().unwrap(),
    };
    let publication = complete(&pool, &request, &CancellationToken::new())
        .await
        .unwrap();
    assert_eq!(publication.data["tags"], json!(["poster"]));
    assert_eq!(
        publication.data["visualKnowledge"]["description"],
        "A red poster"
    );
    assert_eq!(
        publication.data["visualKnowledge"]["sourceRevision"],
        visual_revision(&publication.data)
            .unwrap()
            .unwrap()
            .as_str()
    );
    let queued: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM spaces_enrichment_jobs WHERE node_id='output'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(queued, 0, "a kept reading is not taken again");
}

#[tokio::test]
async fn a_depth_map_is_a_new_work_and_keeps_nothing_of_its_source() {
    let (pool, request, mut source) = fixture().await;
    source["tags"] = json!(["poster"]);
    sqlx::query("UPDATE spaces_nodes SET data=$1 WHERE id='source'")
        .bind(source.to_string())
        .execute(&pool)
        .await
        .unwrap();
    let publication = complete(&pool, &request, &CancellationToken::new())
        .await
        .unwrap();
    assert!(publication.data.get("tags").is_none());
}

async fn assert_unpublished(pool: &SqlitePool) {
    let stored: String = sqlx::query_scalar("SELECT data FROM spaces_nodes WHERE id='output'")
        .fetch_one(pool)
        .await
        .unwrap();
    let payload: Value = serde_json::from_str(&stored).unwrap();
    assert_eq!(payload["assetPath"], "/owned/pending.png");
    assert_eq!(payload["processingState"], "processing");
    let state: String =
        sqlx::query_scalar("SELECT status FROM spaces_knowledge_derivations WHERE id='job'")
            .fetch_one(pool)
            .await
            .unwrap();
    assert_eq!(state, "running");
    let relation_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM spaces_relations")
        .fetch_one(pool)
        .await
        .unwrap();
    assert_eq!(relation_count, 0);
}
