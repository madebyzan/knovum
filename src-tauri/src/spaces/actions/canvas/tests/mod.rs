use super::*;
use crate::app_error::AppError;
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
    target: Option<TaskTargetRef>,
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
                knowledge_revision: (*revision).into(),
            })
            .collect(),
        target,
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
async fn stale_or_malformed_acceptance_rolls_back_grant_and_run() {
    let pool = database().await;
    let revision = node(
        &pool,
        "doc",
        json!({"kind":"markdown","title":"D","text":"Content"}),
    )
    .await;
    let malformed = request(
        "convert-document-to-docx",
        &[("doc", &revision)],
        json!({"invalid":true}),
        None,
    );
    assert!(start(&pool, &malformed)
        .await
        .unwrap_err()
        .message
        .contains("parameters are invalid"));
    let stale = request(
        "convert-document-to-docx",
        &[("doc", "old")],
        json!({}),
        None,
    );
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
    let target = Some(TaskTargetRef::Agent {
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

mod chain;
