use serde_json::Value;
use sha2::{Digest, Sha256};
use sqlx::{Row, SqlitePool};

use super::persistence;
use crate::spaces::{
    authorization::{create_grant, AuthorizationContext, Principal, PrincipalKind},
    db,
    runs::{create_run_in_transaction, NewRun, RunKind, RunSummary},
    workflows::{definition::WorkflowNode, WorkflowDefinition},
};

pub async fn start(
    pool: &SqlitePool,
    space_id: &str,
    run_id: &str,
    template_id: &str,
    definition: WorkflowDefinition,
    input: Value,
) -> Result<RunSummary, String> {
    uuid::Uuid::parse_str(run_id).map_err(|_| "workflow-run-invalid: request ID must be a UUID")?;
    if space_id.is_empty() || template_id.is_empty() {
        return Err("workflow-run-invalid: Space and recipe are required".into());
    }
    persistence::validate_input_budget(&input, definition.limits.max_input_bytes)?;
    let plan = super::super::compile(&definition).map_err(|diagnostics| {
        serde_json::to_string(&diagnostics).expect("workflow diagnostics must serialize")
    })?;
    let digest = format!(
        "{:x}",
        Sha256::digest(serde_json::to_vec(&definition).map_err(db::json_error)?)
    );
    let now = crate::clock::unix_time_millis()?;
    let mut tx = db::begin_write(pool).await?;
    if let Some(row) = sqlx::query(
        "SELECT home_space_id,policy_snapshot,input_snapshot FROM spaces_runs WHERE id=$1",
    )
    .bind(run_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(db::sql_error)?
    {
        let home_space_id: String = row.try_get("home_space_id").map_err(db::sql_error)?;
        let policy: Value = serde_json::from_str(
            &row.try_get::<String, _>("policy_snapshot")
                .map_err(db::sql_error)?,
        )
        .map_err(db::json_error)?;
        let previous_input: Value = serde_json::from_str(
            &row.try_get::<String, _>("input_snapshot")
                .map_err(db::sql_error)?,
        )
        .map_err(db::json_error)?;
        if home_space_id != space_id
            || policy.get("builtinTemplateId").and_then(Value::as_str) != Some(template_id)
            || policy.get("definitionDigest").and_then(Value::as_str) != Some(digest.as_str())
            || previous_input != input
        {
            return Err("workflow-run-conflict: request ID belongs to another Run".into());
        }
        tx.commit().await.map_err(db::sql_error)?;
        return crate::spaces::runs::load_run(pool, run_id).await;
    }
    let grant = user_grant(space_id, &definition.nodes, now);
    create_grant(&mut tx, &grant, now).await?;
    let policy = serde_json::json!({
        "grantId": grant.grant_id,
        "builtinTemplateId": template_id,
        "definitionDigest": digest,
        "definition": &definition,
        "targets": input.get("targets"),
    });
    let budget = serde_json::to_value(&definition.limits).map_err(db::json_error)?;
    create_run_in_transaction(
        &mut tx,
        &NewRun {
            id: run_id,
            home_space_id: space_id,
            parent_run_id: None,
            principal: &grant.principal,
            kind: RunKind::Workflow,
            policy_snapshot: &policy,
            input_snapshot: &input,
            budget: &budget,
        },
        now,
    )
    .await?;
    sqlx::query("UPDATE spaces_runs SET expanded_steps=$1 WHERE id=$2")
        .bind(
            i64::try_from(definition.nodes.len())
                .map_err(|_| "workflow-run-invalid: too many steps")?,
        )
        .bind(run_id)
        .execute(&mut *tx)
        .await
        .map_err(db::sql_error)?;
    persistence::insert_steps(
        &mut tx,
        run_id,
        persistence::StepPlan {
            nodes: &definition.nodes,
            edges: &definition.edges,
            order: &plan.ordered_node_ids,
            input: &input,
        },
        now,
    )
    .await?;
    tx.commit().await.map_err(db::sql_error)?;
    crate::spaces::runs::announce_change();
    crate::spaces::runs::load_run(pool, run_id).await
}

fn user_grant(space_id: &str, nodes: &[WorkflowNode], now: i64) -> AuthorizationContext {
    AuthorizationContext {
        principal: Principal {
            kind: PrincipalKind::User,
            id: "local-user".into(),
        },
        grant_id: uuid::Uuid::new_v4().to_string(),
        home_space_id: space_id.into(),
        read_space_ids: vec![space_id.into()],
        write_space_ids: vec![space_id.into()],
        action_ids: nodes
            .iter()
            .filter_map(|node| match node {
                WorkflowNode::Action { action_id, .. } => Some(action_id.clone()),
                _ => None,
            })
            .collect(),
        valid_until: now + 24 * 60 * 60 * 1_000,
    }
}
