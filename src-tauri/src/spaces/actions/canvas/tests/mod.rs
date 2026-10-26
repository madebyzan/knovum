use super::*;
use crate::foundation::app_error::AppError;
use crate::spaces::derivation::input_revision::RevisionBasis;
use crate::spaces::knowledge::model::revision;

pub(super) async fn database() -> SqlitePool {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    db::migrate(&pool).await.unwrap();
    sqlx::query("INSERT INTO spaces(id,kind,title,created_at,updated_at) VALUES('space','space','Space',1,1)").execute(&pool).await.unwrap();
    pool
}
pub(super) async fn node(pool: &SqlitePool, id: &str, mut data: Value) -> String {
    revision::apply(&mut data).unwrap();
    let revision = data["knowledgeRevision"].as_str().unwrap().to_owned();
    sqlx::query("INSERT INTO spaces_nodes(id,space_id,kind,data,initial_x,initial_y,created_at,updated_at) VALUES($1,'space',$2,$3,0,0,1,1)")
        .bind(id).bind(data["kind"].as_str().unwrap()).bind(data.to_string()).execute(pool).await.unwrap();
    revision
}
pub(super) fn request(
    action: &str,
    sources: &[(&str, &str)],
    parameters: Value,
    target: Option<TaskTarget>,
) -> StartSpaceActionRequest {
    pinned_request(
        action,
        sources,
        RevisionBasis::Knowledge,
        parameters,
        target,
    )
}
pub(super) fn pinned_request(
    action: &str,
    sources: &[(&str, &str)],
    revision_basis: RevisionBasis,
    parameters: Value,
    target: Option<TaskTarget>,
) -> StartSpaceActionRequest {
    StartSpaceActionRequest {
        request_id: uuid::Uuid::new_v4().to_string(),
        space_id: "space".into(),
        action_id: action.into(),
        parameters,
        sources: sources
            .iter()
            .map(|(id, revision)| SpaceActionSource {
                node_id: (*id).into(),
                source_revision: (*revision).into(),
                revision_basis,
            })
            .collect(),
        target,
        output_frame: None,
    }
}
pub(super) async fn start(
    pool: &SqlitePool,
    request: &StartSpaceActionRequest,
) -> Result<RunObservation, AppError> {
    accept(
        pool,
        request,
        invocation(request, super::super::require(&request.action_id)?)?,
    )
    .await
}
#[tokio::test]
async fn local_video_action_has_one_idempotent_run_and_no_intermediate_resources() {
    let pool = database().await;
    let revision = node(
        &pool,
        "video",
        json!({"kind":"video","title":"V","sourceRevision":"asset:1","assetPath":"/owned/v.mp4"}),
    )
    .await;
    let request = request(
        "add-video-subtitles",
        &[("video", &revision)],
        json!({}),
        None,
    );
    let run = start(&pool, &request).await.unwrap();
    assert_eq!(run.kind, crate::spaces::runs::RunKind::Action);
    assert_eq!(start(&pool, &request).await.unwrap().id, run.id);
    let counts: (i64,i64,i64) = sqlx::query_as("SELECT (SELECT COUNT(*) FROM spaces_runs),(SELECT COUNT(*) FROM spaces_authorization_grants),(SELECT COUNT(*) FROM spaces_nodes)").fetch_one(&pool).await.unwrap();
    assert_eq!(counts, (1, 1, 1));
    let input: String = sqlx::query_scalar("SELECT input_snapshot FROM spaces_runs WHERE id=$1")
        .bind(run.id)
        .fetch_one(&pool)
        .await
        .unwrap();
    let input: Value = serde_json::from_str(&input).unwrap();
    assert_eq!(input["inputs"]["targets"], json!({}));
    assert_eq!(
        input["inputs"]["orderedInputs"][0]["revisionBasis"],
        "knowledge"
    );
}
#[tokio::test]
async fn an_unanalyzed_image_starts_a_creation_run_pinned_on_its_pixels() {
    let pool = database().await;
    node(
        &pool,
        "image",
        json!({"kind":"image","title":"Shot","sourceRevision":"asset:1","assetPath":"/owned/shot.png","fileName":"shot.png"}),
    )
    .await;
    let request = pinned_request(
        "generate-visualization",
        &[("image", "asset:1")],
        RevisionBasis::Asset,
        json!({"title":"Variant","intent":"Make it snow","aspectRatio":"1:1"}),
        Some(TaskTarget::Agent {
            runtime_kind: crate::ai::contracts::ExternalAgentKind::Codex,
        }),
    );
    let run = start(&pool, &request).await.unwrap();
    let input: String = sqlx::query_scalar("SELECT input_snapshot FROM spaces_runs WHERE id=$1")
        .bind(run.id)
        .fetch_one(&pool)
        .await
        .unwrap();
    let input: Value = serde_json::from_str(&input).unwrap();
    assert_eq!(
        input["inputs"]["orderedInputs"][0]["revisionBasis"], "asset",
        "an image with no visual knowledge is still a resource to create from"
    );
}

#[tokio::test]
async fn stale_or_malformed_acceptance_rolls_back_grant_and_run() {
    let pool = database().await;
    let revision = node(
        &pool,
        "doc",
        json!({"kind":"markdown","title":"D","text":"Content"}),
    )
    .await;
    let malformed = request(
        "add-video-subtitles",
        &[("doc", &revision)],
        json!({"invalid":true}),
        None,
    );
    assert!(start(&pool, &malformed)
        .await
        .unwrap_err()
        .message
        .contains("parameters are invalid"));
    let stale = request("add-video-subtitles", &[("doc", "old")], json!({}), None);
    assert!(start(&pool, &stale)
        .await
        .unwrap_err()
        .message
        .contains("action-input-stale"));
    let counts: (i64,i64) = sqlx::query_as("SELECT (SELECT COUNT(*) FROM spaces_runs),(SELECT COUNT(*) FROM spaces_authorization_grants)").fetch_one(&pool).await.unwrap();
    assert_eq!(counts, (0, 0));
}

#[test]
fn local_actions_reject_ai_targets_and_immediate_mutations() {
    let target = Some(TaskTarget::Agent {
        runtime_kind: crate::ai::contracts::ExternalAgentKind::Codex,
    });
    let request = request(
        "add-video-subtitles",
        &[("v", "revision")],
        json!({}),
        target,
    );
    assert!(invocation(&request, super::super::require(&request.action_id).unwrap()).is_err());
    let request = StartSpaceActionRequest {
        action_id: "create-note".into(),
        target: None,
        ..request
    };
    assert!(invocation(&request, super::super::require(&request.action_id).unwrap()).is_err());
}

#[tokio::test]
async fn a_generation_starts_from_its_prompt_alone_and_keeps_the_frame_it_was_drawn_in() {
    let pool = database().await;
    let frame = json!({"x": 120.0, "y": -40.0, "width": 320.0, "height": 180.0});
    let request = StartSpaceActionRequest {
        output_frame: Some(serde_json::from_value(frame.clone()).unwrap()),
        ..request(
            "generate-visualization",
            &[],
            json!({"title":"Harbour","intent":"A harbour at dawn","aspectRatio":"16:9"}),
            Some(TaskTarget::Agent {
                runtime_kind: crate::ai::contracts::ExternalAgentKind::Codex,
            }),
        )
    };
    let run = start(&pool, &request).await.unwrap();
    let input: String = sqlx::query_scalar("SELECT input_snapshot FROM spaces_runs WHERE id=$1")
        .bind(run.id)
        .fetch_one(&pool)
        .await
        .unwrap();
    let input: Value = serde_json::from_str(&input).unwrap();
    assert_eq!(input["inputs"]["orderedInputs"], json!([]));
    assert_eq!(input["inputs"]["outputFrame"], frame);
}

#[test]
fn an_action_that_reads_a_resource_refuses_to_start_without_one() {
    let request = request("add-video-subtitles", &[], json!({}), None);
    let error = invocation(&request, super::super::require(&request.action_id).unwrap())
        .err()
        .unwrap();
    assert!(error.message.contains("action-input-missing"));
}

#[test]
fn an_output_frame_must_be_a_real_rectangle() {
    let request = StartSpaceActionRequest {
        output_frame: Some(
            serde_json::from_value(json!({"x":0.0,"y":0.0,"width":0.0,"height":10.0})).unwrap(),
        ),
        ..request(
            "generate-visualization",
            &[],
            json!({"title":"T","intent":"I","aspectRatio":"1:1"}),
            None,
        )
    };
    let error = invocation(&request, super::super::require(&request.action_id).unwrap())
        .err()
        .unwrap();
    assert!(error.message.contains("canvas-placement-invalid"));
}
