use serde_json::Value;
use sqlx::{Row, Sqlite, Transaction};

use crate::spaces::{
    db,
    runs::{run_rows::enum_text, RunState},
};

mod branches;
mod dependencies;

pub(super) use branches::skip_unselected_branches;
pub(super) use dependencies::resolve_dependents;

async fn dependency_facts(
    tx: &mut Transaction<'_, Sqlite>,
    run_id: &str,
    dependency_step_ids: &[String],
) -> Result<Vec<(String, Option<Value>)>, String> {
    let mut facts = Vec::with_capacity(dependency_step_ids.len());
    for id in dependency_step_ids {
        let row = sqlx::query(
            "SELECT state,output FROM spaces_run_steps WHERE run_id=$1 AND definition_node_id=$2",
        )
        .bind(run_id)
        .bind(id)
        .fetch_one(&mut **tx)
        .await
        .map_err(db::sql_error)?;
        facts.push((
            row.try_get("state").map_err(db::sql_error)?,
            row.try_get::<Option<sqlx::types::Json<Value>>, _>("output")
                .map_err(db::sql_error)?
                .map(|output| output.0),
        ));
    }
    Ok(facts)
}

fn dependency_input(
    dependency_step_ids: &[String],
    facts: &[(String, Option<Value>)],
    include_states: bool,
) -> Value {
    let values = facts
        .iter()
        .map(|(_, output)| output.clone().unwrap_or(Value::Null))
        .collect::<Vec<_>>();
    if include_states {
        Value::Array(
            dependency_step_ids
                .iter()
                .zip(facts)
                .zip(values)
                .map(|((id, (state, _)), output)| {
                    serde_json::json!({
                        "stepId": id, "state": state, "output": output,
                    })
                })
                .collect(),
        )
    } else if values.len() == 1 {
        values
            .into_iter()
            .next()
            .expect("single dependency has one value")
    } else {
        Value::Object(dependency_step_ids.iter().cloned().zip(values).collect())
    }
}

fn join_policy(configuration: &str) -> Result<String, String> {
    let configuration: Value = serde_json::from_str(configuration).map_err(db::json_error)?;
    Ok(configuration
        .get("policy")
        .and_then(Value::as_str)
        .unwrap_or("all_succeeded")
        .to_owned())
}

fn next_state(states: &[String], collect_errors: bool) -> Option<&'static str> {
    if states.iter().any(|state| {
        matches!(
            state.as_str(),
            "pending" | "ready" | "running" | "waiting" | "paused"
        )
    }) {
        None
    } else if states
        .iter()
        .any(|state| matches!(state.as_str(), "failed" | "blocked" | "cancelled"))
    {
        if collect_errors {
            Some("ready")
        } else {
            Some("blocked")
        }
    } else if states.iter().all(|state| state == "skipped") {
        Some("skipped")
    } else {
        Some("ready")
    }
}

pub(super) async fn resolve_run(
    tx: &mut Transaction<'_, Sqlite>,
    run_id: &str,
    now: i64,
) -> Result<(), String> {
    let states =
        sqlx::query_scalar::<_, String>("SELECT state FROM spaces_run_steps WHERE run_id=$1")
            .bind(run_id)
            .fetch_all(&mut **tx)
            .await
            .map_err(db::sql_error)?;
    if states.iter().any(|state| {
        matches!(
            state.as_str(),
            "pending" | "ready" | "running" | "waiting" | "paused"
        )
    }) {
        return Ok(());
    }
    let has_errors = states
        .iter()
        .any(|state| matches!(state.as_str(), "failed" | "blocked"));
    let policy = run_failure_policy(tx, run_id).await?;
    let state = if has_errors && policy == "collect_errors" {
        RunState::CompletedWithErrors
    } else if has_errors {
        RunState::Failed
    } else {
        RunState::Succeeded
    };
    let changed = sqlx::query("UPDATE spaces_runs SET state=$1,state_version=state_version+1,wait_reason=NULL,updated_at=$2,completed_at=$2 WHERE id=$3 AND cancel_requested_at IS NULL AND completed_at IS NULL")
        .bind(enum_text(state)?).bind(now).bind(run_id).execute(&mut **tx).await.map_err(db::sql_error)?;
    db::require_one(
        changed,
        "run-stale: Run authority changed before resolution",
    )
}

async fn run_failure_policy(
    tx: &mut Transaction<'_, Sqlite>,
    run_id: &str,
) -> Result<String, String> {
    let encoded: String = sqlx::query_scalar("SELECT policy_snapshot FROM spaces_runs WHERE id=$1")
        .bind(run_id)
        .fetch_one(&mut **tx)
        .await
        .map_err(db::sql_error)?;
    let value: Value = serde_json::from_str(&encoded).map_err(db::json_error)?;
    Ok(failure_policy(&value).to_owned())
}

pub(super) fn failure_policy(policy: &Value) -> &str {
    policy
        .get("definition")
        .and_then(|value| value.get("failurePolicy"))
        .and_then(Value::as_str)
        .unwrap_or("stop_run")
}
