use sqlx::SqlitePool;

use crate::spaces::{
    authorization::create_grant,
    db,
    runs::{create_run_in_transaction, NewRun, RunKind, RunSummary},
};

use super::{
    contract::{StartWorkflowRequest, StartWorkflowStepTestRequest},
    persistence,
    start::user_grant,
};
use crate::spaces::workflows::repository;

pub async fn start_step_test(
    pool: &SqlitePool,
    request: StartWorkflowStepTestRequest,
) -> Result<RunSummary, String> {
    let workflow = repository::load_version(pool, &request.workflow_id, request.version).await?;
    if workflow.space_id != request.space_id {
        return Err("permission-denied: Workflow belongs to another Space".into());
    }
    persistence::validate_input_budget(&request.input, workflow.definition.limits.max_input_bytes)?;
    let node = workflow
        .definition
        .nodes
        .iter()
        .find(|node| node.id() == request.node_id)
        .cloned()
        .ok_or("workflow-node-unavailable: selected test step is unavailable")?;
    let now = crate::clock::unix_time_millis()?;
    let run_id = uuid::Uuid::new_v4().to_string();
    let start_request = StartWorkflowRequest {
        space_id: request.space_id,
        workflow_id: request.workflow_id,
        version: request.version,
        input: request.input,
        parent_run_id: None,
        grant_id: None,
        test_mode: true,
        breakpoint_node_ids: Vec::new(),
    };
    let mut tx = db::begin_write(pool).await?;
    let grant = user_grant(&start_request, std::slice::from_ref(&node), now);
    create_grant(&mut tx, &grant, now).await?;
    let policy = serde_json::json!({
        "grantId": grant.grant_id,
        "workflowId": start_request.workflow_id,
        "workflowVersion": start_request.version,
        "definitionDigest": workflow.definition_digest,
        "definition": workflow.definition,
        "testMode": true,
        "isolatedStepNodeId": node.id(),
        "breakpointNodeIds": [],
    });
    let budget = serde_json::to_value(&workflow.definition.limits).map_err(db::json_error)?;
    create_run_in_transaction(
        &mut tx,
        &NewRun {
            id: &run_id,
            home_space_id: &start_request.space_id,
            parent_run_id: None,
            principal: &grant.principal,
            kind: RunKind::Workflow,
            policy_snapshot: &policy,
            input_snapshot: &start_request.input,
            budget: &budget,
        },
        now,
    )
    .await?;
    let order = [node.id().to_owned()];
    persistence::insert_steps(
        &mut tx,
        &run_id,
        persistence::StepPlan {
            nodes: std::slice::from_ref(&node),
            edges: &[],
            order: &order,
            input: &start_request.input,
            breakpoint_node_ids: &[],
        },
        now,
    )
    .await?;
    tx.commit().await.map_err(db::sql_error)?;
    crate::spaces::runs::load_run(pool, &run_id).await
}
