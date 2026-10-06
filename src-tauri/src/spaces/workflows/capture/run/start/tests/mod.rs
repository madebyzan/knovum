use super::*;
use crate::ai::{contracts::ExternalAgentKind, tasks::TaskTargetSelection};
use crate::spaces::{
    knowledge::indexing::revision,
    runs::{AttemptClaim, StepOutcome},
    workflows::capture::contracts::*,
};
use serde_json::Value;

async fn database() -> SqlitePool {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    db::migrate(&pool).await.unwrap();
    sqlx::query("INSERT INTO spaces(id,kind,title,created_at,updated_at) VALUES('space','space','Space',1,1)").execute(&pool).await.unwrap();
    insert_node(&pool,"new-note",json!({"kind":"note","title":"New idea","plainText":"New campaign","tiptapContent":{"type":"doc","content":[]}})).await;
    pool
}
async fn insert_node(pool: &SqlitePool, id: &str, mut data: Value) {
    revision::apply(&mut data).unwrap();
    sqlx::query("INSERT INTO spaces_nodes(id,space_id,kind,data,initial_x,initial_y,created_at,updated_at) VALUES($1,'space',$2,$3,0,0,1,1)")
        .bind(id).bind(data["kind"].as_str().unwrap()).bind(data.to_string()).execute(pool).await.unwrap();
}
fn workflow() -> SavedCapturedWorkflow {
    let binding = |source| CandidateBinding {
        role: CaptureInputRole::Primary,
        position: 0,
        revision_basis: crate::spaces::derivation::input_revision::RevisionBasis::Knowledge,
        source,
    };
    SavedCapturedWorkflow {
        schema_version: 3,
        workflow_id: "saved".into(),
        version: 1,
        name: "Report".into(),
        source_space_id: "original-space".into(),
        source_digest: "a".repeat(64),
        selected_node_ids: vec!["old-output".into()],
        created_at: 1,
        candidate: CaptureCandidate {
            input_slots: vec![CandidateInputSlot {
                id: "idea".into(),
                kind: "note".into(),
                revision_bases: vec![
                    crate::spaces::derivation::input_revision::RevisionBasis::Knowledge,
                ],
                min_items: 1,
                max_items: 1,
                ordered: false,
            }],
            steps: vec![
                CandidateStep {
                    id: "draft".into(),
                    action_id: "draft-document".into(),
                    action_version: 1,
                    output_kind: "markdown".into(),
                    execution_spec: RecordedExecutionSpec {
                        parameters: json!({"title":"Report","purpose":"Summarize"}),
                        target: Some(TaskTargetSelection::Agent {
                            runtime_kind: ExternalAgentKind::Codex,
                        }),
                    },
                    inputs: vec![binding(CandidateSource::InputSlot {
                        input_slot_id: "idea".into(),
                    })],
                },
                CandidateStep {
                    id: "export".into(),
                    action_id: "export-document-as-docx".into(),
                    action_version: 1,
                    output_kind: "file".into(),
                    execution_spec: RecordedExecutionSpec {
                        parameters: json!({}),
                        target: None,
                    },
                    inputs: vec![binding(CandidateSource::StepOutput {
                        step_id: "draft".into(),
                    })],
                },
            ],
            output_slots: vec![CandidateOutputSlot {
                id: "report".into(),
                name: "Report.docx".into(),
                kind: "file".into(),
                step_id: "export".into(),
            }],
        },
    }
}
async fn create_run(pool: &SqlitePool) -> String {
    let saved = workflow();
    validation::candidate(&saved.candidate).unwrap();
    let request = StartCapturedWorkflowRequest {
        request_id: uuid::Uuid::new_v4().to_string(),
        workflow_id: saved.workflow_id.clone(),
        expected_version: 1,
        space_id: "space".into(),
        bindings: std::collections::HashMap::from([("idea".into(), vec!["new-note".into()])]),
    };
    let mut tx = db::begin_write(pool).await.unwrap();
    create(&mut tx, &request, &saved).await.unwrap();
    tx.commit().await.unwrap();
    request.request_id
}
async fn claim(pool: &SqlitePool, run_id: &str, step: &str) -> AttemptClaim {
    let (step_id, configuration): (String, String) = sqlx::query_as(
        "SELECT id,configuration FROM spaces_run_steps WHERE run_id=$1 AND definition_node_id=$2",
    )
    .bind(run_id)
    .bind(step)
    .fetch_one(pool)
    .await
    .unwrap();
    let policy: String = sqlx::query_scalar("SELECT policy_snapshot FROM spaces_runs WHERE id=$1")
        .bind(run_id)
        .fetch_one(pool)
        .await
        .unwrap();
    AttemptClaim {
        attempt_id: uuid::Uuid::new_v4().to_string(),
        run_id: run_id.into(),
        step_instance_id: step_id,
        execution_epoch: 0,
        attempt_number: 1,
        home_space_id: "space".into(),
        node_kind: "captured_action".into(),
        configuration: serde_json::from_str(&configuration).unwrap(),
        resolved_input: json!({}),
        policy_snapshot: serde_json::from_str(&policy).unwrap(),
        budget: json!({}),
        cancellation: tokio_util::sync::CancellationToken::new(),
    }
}
async fn child(pool: &SqlitePool, claim: &AttemptClaim) -> String {
    let StepOutcome::Waiting { checkpoint, .. } = super::super::super::execute_step(pool, claim)
        .await
        .unwrap()
    else {
        panic!("Expected a child Action Run")
    };
    checkpoint["childRunId"].as_str().unwrap().to_owned()
}
#[tokio::test]
async fn replay_binds_new_input_freezes_agent_and_chains_real_action_runs() {
    let pool = database().await;
    let run_id = create_run(&pool).await;
    let states: Vec<(String,String)> = sqlx::query_as("SELECT definition_node_id,state FROM spaces_run_steps WHERE run_id=$1 ORDER BY definition_node_id").bind(&run_id).fetch_all(&pool).await.unwrap();
    assert_eq!(
        states,
        vec![
            ("draft".into(), "ready".into()),
            ("export".into(), "pending".into())
        ]
    );
    let first = claim(&pool, &run_id, "draft").await;
    let child_id = child(&pool, &first).await;
    let input: String = sqlx::query_scalar("SELECT input_snapshot FROM spaces_runs WHERE id=$1")
        .bind(&child_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    let input: Value = serde_json::from_str(&input).unwrap();
    assert_eq!(input["inputs"]["orderedInputs"][0]["nodeId"], "new-note");
    assert_eq!(
        input["inputs"]["targets"]["draft-document"]["runtimeKind"],
        "codex"
    );
    assert!(!input.to_string().contains("connectionId"));
    // Inference is simulated; dependency binding and child acceptance use production code.
    insert_node(
        &pool,
        "new-document",
        json!({"kind":"markdown","title":"New report","text":"New campaign report"}),
    )
    .await;
    sqlx::query("INSERT INTO spaces_knowledge_derivations(id,space_id,output_node_id,action_kind,action_version,engine,status,created_at,updated_at,completed_at) VALUES($1,'space','new-document','draft-document',1,'fixture','completed',1,2,2)").bind(&child_id).execute(&pool).await.unwrap();
    sqlx::query("UPDATE spaces_runs SET state='succeeded',completed_at=2 WHERE id=$1")
        .bind(&child_id)
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("UPDATE spaces_run_steps SET state='succeeded',output=$1 WHERE id=$2")
        .bind(json!({"childRunId":child_id}).to_string())
        .bind(&first.step_instance_id)
        .execute(&pool)
        .await
        .unwrap();
    let second = claim(&pool, &run_id, "export").await;
    let export_id = child(&pool, &second).await;
    let input: String = sqlx::query_scalar("SELECT input_snapshot FROM spaces_runs WHERE id=$1")
        .bind(export_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    let input: Value = serde_json::from_str(&input).unwrap();
    assert_eq!(
        input["inputs"]["orderedInputs"][0]["nodeId"],
        "new-document"
    );
    assert!(input["inputs"].get("targets").is_none());
}
#[tokio::test]
async fn edited_inputs_and_cancelled_parents_cannot_create_child_runs() {
    let pool = database().await;
    let cancelled = create_run(&pool).await;
    crate::spaces::runs::request_cancellation(&pool, &cancelled)
        .await
        .unwrap();
    let error = super::super::super::execute_step(&pool, &claim(&pool, &cancelled, "draft").await)
        .await
        .unwrap_err();
    assert!(error.message.contains("parent-run-inactive"));
    let stale = create_run(&pool).await;
    sqlx::query(
        "UPDATE spaces_nodes SET data=json_set(data,'$.title','Edited') WHERE id='new-note'",
    )
    .execute(&pool)
    .await
    .unwrap();
    let error = super::super::super::execute_step(&pool, &claim(&pool, &stale, "draft").await)
        .await
        .unwrap_err();
    assert!(error.message.contains("workflow-run-stale"));
    let count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM spaces_runs WHERE parent_run_id IS NOT NULL")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(count, 0);
}
#[test]
fn replay_rejects_cycles_and_unregistered_execution_contracts() {
    let mut saved = workflow();
    saved.candidate.steps[0].inputs[0].source = CandidateSource::StepOutput {
        step_id: "export".into(),
    };
    assert!(validation::candidate(&saved.candidate).is_err());
    let mut saved = workflow();
    saved.candidate.steps[0].execution_spec.target = None;
    assert!(validation::candidate(&saved.candidate).is_err());
}
