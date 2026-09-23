use std::collections::{HashMap, HashSet, VecDeque};

use serde_json::Value;
use sqlx::SqlitePool;

use super::RunSummary;
use crate::spaces::db;

mod source;

use source::{load_source, load_steps, SourceStep};

pub(super) async fn create_retry(
    pool: &SqlitePool,
    source_run_id: &str,
    selected_step_ids: &[String],
) -> Result<RunSummary, String> {
    let source = load_source(pool, source_run_id).await?;
    let steps = load_steps(pool, source_run_id).await?;
    let selected = selected_nodes(&steps, selected_step_ids)?;
    let affected = affected_nodes(&source.policy, &steps, selected)?;
    let now = crate::clock::unix_time_millis()?;
    let run_id = uuid::Uuid::new_v4().to_string();
    let retry_policy = retry_policy(&source.policy, source_run_id)?;
    let mut tx = db::begin_write(pool).await?;
    sqlx::query(
        "INSERT INTO spaces_runs(id,home_space_id,parent_run_id,retry_of_run_id,principal_kind,
         principal_id,kind,state,state_version,execution_epoch,policy_snapshot,input_snapshot,budget,
         created_at,updated_at) VALUES($1,$2,NULL,$3,$4,$5,$6,'queued',0,0,$7,$8,$9,$10,$10)",
    )
    .bind(&run_id).bind(&source.space_id).bind(source_run_id)
    .bind(&source.principal_kind).bind(&source.principal_id).bind(&source.kind)
    .bind(retry_policy).bind(&source.input).bind(&source.budget).bind(now)
    .execute(&mut *tx).await.map_err(db::sql_error)?;
    for step in &steps {
        insert_step(&mut tx, &run_id, step, &affected, now).await?;
    }
    super::append_event(
        &mut tx,
        &run_id,
        None,
        None,
        crate::spaces::runs::RunEventKind::RunRetryCreated,
        &serde_json::json!({"retryOfRunId": source_run_id, "affectedNodeIds": affected}),
        now,
    )
    .await?;
    tx.commit().await.map_err(db::sql_error)?;
    super::repository::load_run(pool, &run_id).await
}

fn retry_policy(encoded: &str, source_run_id: &str) -> Result<String, String> {
    let mut policy: serde_json::Map<String, Value> =
        serde_json::from_str(encoded).map_err(db::json_error)?;
    policy
        .entry("retryRootRunId")
        .or_insert_with(|| Value::String(source_run_id.to_owned()));
    serde_json::to_string(&policy).map_err(db::json_error)
}

fn selected_nodes(
    steps: &[SourceStep],
    selected_step_ids: &[String],
) -> Result<HashSet<String>, String> {
    if selected_step_ids.is_empty() {
        return Ok(steps
            .iter()
            .filter(|step| matches!(step.state.as_str(), "failed" | "blocked" | "cancelled"))
            .map(|step| step.node_id.clone())
            .collect());
    }
    let by_id = steps
        .iter()
        .map(|step| (&step.id, &step.node_id))
        .collect::<HashMap<_, _>>();
    selected_step_ids
        .iter()
        .map(|id| {
            by_id
                .get(id)
                .map(|node| (*node).clone())
                .ok_or_else(|| "run-retry-invalid: selected Step is not part of the Run".into())
        })
        .collect()
}

fn affected_nodes(
    policy: &str,
    steps: &[SourceStep],
    selected: HashSet<String>,
) -> Result<HashSet<String>, String> {
    if selected.is_empty() {
        return Err("run-retry-invalid: Run has no failed or selected steps to retry".into());
    }
    let policy: Value = serde_json::from_str(policy).map_err(db::json_error)?;
    let edges = policy
        .pointer("/definition/edges")
        .and_then(Value::as_array);
    let mut children = HashMap::<String, Vec<String>>::new();
    if let Some(edges) = edges {
        for edge in edges {
            if let (Some(source), Some(target)) = (
                edge.get("sourceNodeId").and_then(Value::as_str),
                edge.get("targetNodeId").and_then(Value::as_str),
            ) {
                children
                    .entry(source.to_owned())
                    .or_default()
                    .push(target.to_owned());
            }
        }
    } else if steps.len() > 1 {
        return Err("run-retry-invalid: Run has no dependency graph for selective retry".into());
    }
    let mut affected = selected;
    let mut queue = affected.iter().cloned().collect::<VecDeque<_>>();
    while let Some(node) = queue.pop_front() {
        for child in children.get(&node).into_iter().flatten() {
            if affected.insert(child.clone()) {
                queue.push_back(child.clone());
            }
        }
    }
    Ok(affected)
}

async fn insert_step(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    run_id: &str,
    step: &SourceStep,
    affected: &HashSet<String>,
    now: i64,
) -> Result<(), String> {
    let dependencies: Vec<String> =
        serde_json::from_str(&step.depends_on).map_err(db::json_error)?;
    let rerun = affected.contains(&step.node_id);
    let state = if rerun {
        if dependencies
            .iter()
            .any(|dependency| affected.contains(dependency))
        {
            "pending"
        } else {
            "ready"
        }
    } else if matches!(step.state.as_str(), "succeeded" | "skipped") {
        &step.state
    } else {
        "blocked"
    };
    let output = step.output.as_deref();
    sqlx::query(
        "INSERT INTO spaces_run_steps(id,run_id,definition_node_id,instance_path,node_kind,depends_on,
         configuration,state,state_version,resolved_input,output,created_at,updated_at)
         VALUES($1,$2,$3,$4,$5,$6,$7,$8,0,$9,$10,$11,$11)",
    ).bind(uuid::Uuid::new_v4().to_string()).bind(run_id).bind(&step.node_id)
        .bind(&step.instance_path).bind(&step.node_kind).bind(&step.depends_on)
        .bind(&step.configuration).bind(state).bind(&step.input).bind(output).bind(now)
        .execute(&mut **tx).await.map_err(db::sql_error)?;
    Ok(())
}
