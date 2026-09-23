use serde_json::Value;
use sqlx::{Row, SqlitePool};

use crate::spaces::{
    authorization::{create_grant, AuthorizationContext, Principal, PrincipalKind},
    db,
    runs::{create_run_in_transaction, NewRun, RunKind, RunSummary},
};

use super::{contract::StartWorkflowRequest, persistence};
use crate::spaces::workflows::{definition::WorkflowNode, repository};

pub async fn start(pool: &SqlitePool, request: StartWorkflowRequest) -> Result<RunSummary, String> {
    start_with_run_id(pool, request, None).await
}

pub(crate) async fn start_with_run_id(
    pool: &SqlitePool,
    request: StartWorkflowRequest,
    requested_run_id: Option<&str>,
) -> Result<RunSummary, String> {
    let workflow = repository::load_version(pool, &request.workflow_id, request.version).await?;
    if workflow.space_id != request.space_id {
        return Err("permission-denied: Workflow belongs to another Space".into());
    }
    persistence::validate_input_budget(&request.input, workflow.definition.limits.max_input_bytes)?;
    validate_parent_chain(
        pool,
        request.parent_run_id.as_deref(),
        &request.workflow_id,
        request.version,
    )
    .await?;
    let plan = super::super::compile(&workflow.definition).map_err(encode_diagnostics)?;
    let now = crate::clock::unix_time_millis()?;
    let run_id = requested_run_id
        .map(str::to_owned)
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    if requested_run_id.is_some() {
        if let Ok(existing) = crate::spaces::runs::load_run(pool, &run_id).await {
            return Ok(existing);
        }
    }
    let mut tx = db::begin_write(pool).await?;
    if let Some(parent_run_id) = request.parent_run_id.as_deref() {
        let count = i64::try_from(workflow.definition.nodes.len())
            .map_err(|_| "budget-exhausted: Workflow node count overflow")?;
        crate::spaces::runs::reserve_expanded_steps_in_transaction(&mut tx, parent_run_id, count)
            .await?;
    }
    let grant = match request.grant_id.as_deref() {
        Some(grant_id) => crate::spaces::authorization::load_grant(&mut tx, grant_id).await?,
        None => {
            let grant = user_grant(&request, &workflow.definition.nodes, now);
            create_grant(&mut tx, &grant, now).await?;
            grant
        }
    };
    grant.require_read(&request.space_id, now)?;
    let policy = serde_json::json!({
        "grantId": grant.grant_id,
        "workflowId": request.workflow_id,
        "workflowVersion": request.version,
        "definitionDigest": workflow.definition_digest,
        "definition": workflow.definition,
        "testMode": request.test_mode,
        "breakpointNodeIds": request.breakpoint_node_ids,
    });
    let budget = serde_json::to_value(&workflow.definition.limits).map_err(db::json_error)?;
    create_run_in_transaction(
        &mut tx,
        &NewRun {
            id: &run_id,
            home_space_id: &request.space_id,
            parent_run_id: request.parent_run_id.as_deref(),
            principal: &grant.principal,
            kind: RunKind::Workflow,
            policy_snapshot: &policy,
            input_snapshot: &request.input,
            budget: &budget,
        },
        now,
    )
    .await?;
    sqlx::query("UPDATE spaces_runs SET expanded_steps=$1 WHERE id=$2")
        .bind(
            i64::try_from(workflow.definition.nodes.len())
                .map_err(|_| "budget-exhausted: Workflow node count overflow")?,
        )
        .bind(&run_id)
        .execute(&mut *tx)
        .await
        .map_err(db::sql_error)?;
    persistence::insert_steps(
        &mut tx,
        &run_id,
        persistence::StepPlan {
            nodes: &workflow.definition.nodes,
            edges: &workflow.definition.edges,
            order: &plan.ordered_node_ids,
            input: &request.input,
            breakpoint_node_ids: &request.breakpoint_node_ids,
        },
        now,
    )
    .await?;
    persistence::pause_at_initial_breakpoint(&mut tx, &run_id, now).await?;
    tx.commit().await.map_err(db::sql_error)?;
    crate::spaces::runs::load_run(pool, &run_id).await
}

async fn validate_parent_chain(
    pool: &SqlitePool,
    parent_run_id: Option<&str>,
    workflow_id: &str,
    version: i64,
) -> Result<(), String> {
    let Some(mut current) = parent_run_id.map(str::to_owned) else {
        return Ok(());
    };
    for depth in 1..=8 {
        let row = sqlx::query("SELECT parent_run_id,policy_snapshot FROM spaces_runs WHERE id=$1")
            .bind(&current)
            .fetch_optional(pool)
            .await
            .map_err(db::sql_error)?
            .ok_or("parent-run-unavailable: parent Run is unavailable")?;
        let policy: Value = serde_json::from_str(
            &row.try_get::<String, _>("policy_snapshot")
                .map_err(db::sql_error)?,
        )
        .map_err(db::json_error)?;
        if policy.get("workflowId").and_then(Value::as_str) == Some(workflow_id)
            && policy.get("workflowVersion").and_then(Value::as_i64) == Some(version)
        {
            return Err("workflow-recursion-forbidden: active Run ancestry already contains this Workflow version".into());
        }
        let parent: Option<String> = row.try_get("parent_run_id").map_err(db::sql_error)?;
        let Some(parent) = parent else { return Ok(()) };
        if depth == 8 {
            return Err("subworkflow-depth-exceeded: maximum depth is 8".into());
        }
        current = parent;
    }
    Ok(())
}

fn encode_diagnostics(diagnostics: Vec<super::super::WorkflowDiagnostic>) -> String {
    serde_json::to_string(&diagnostics)
        .expect("Workflow diagnostics only contain serializable repository-owned values")
}

pub(super) fn user_grant(
    request: &StartWorkflowRequest,
    nodes: &[WorkflowNode],
    now: i64,
) -> AuthorizationContext {
    let action_ids = nodes
        .iter()
        .filter_map(|node| match node {
            WorkflowNode::Action { action_id, .. } => Some(action_id.clone()),
            _ => None,
        })
        .collect();
    let egress_targets = nodes
        .iter()
        .filter_map(|node| match node {
            WorkflowNode::ExternalTool { binding_id, .. } => {
                Some(format!("external-tool:{binding_id}"))
            }
            _ => None,
        })
        .collect();
    AuthorizationContext {
        principal: Principal {
            kind: PrincipalKind::User,
            id: "local-user".into(),
        },
        grant_id: uuid::Uuid::new_v4().to_string(),
        home_space_id: request.space_id.clone(),
        read_space_ids: vec![request.space_id.clone()],
        write_space_ids: vec![request.space_id.clone()],
        action_ids,
        egress_targets,
        valid_until: now + 24 * 60 * 60 * 1_000,
    }
}
