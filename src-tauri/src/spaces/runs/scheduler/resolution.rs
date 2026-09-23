use serde_json::Value;
use sqlx::{Row, Sqlite, Transaction};

use crate::spaces::{db, runs::RunState};

mod branches;
mod dependencies;

pub(super) use branches::skip_unselected_branches;
pub(super) use dependencies::resolve_dependents;

async fn run_has_breakpoint(
    tx: &mut Transaction<'_, Sqlite>,
    run_id: &str,
    node_id: &str,
) -> Result<bool, String> {
    let policy: String = sqlx::query_scalar("SELECT policy_snapshot FROM spaces_runs WHERE id=$1")
        .bind(run_id)
        .fetch_one(&mut **tx)
        .await
        .map_err(db::sql_error)?;
    let policy: Value = serde_json::from_str(&policy).map_err(db::json_error)?;
    Ok(policy
        .get("breakpointNodeIds")
        .and_then(Value::as_array)
        .is_some_and(|ids| ids.iter().any(|id| id.as_str() == Some(node_id))))
}

async fn dependency_facts(
    tx: &mut Transaction<'_, Sqlite>,
    run_id: &str,
    ids: &[String],
) -> Result<Vec<(String, Option<String>)>, String> {
    let mut facts = Vec::with_capacity(ids.len());
    for id in ids {
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
            row.try_get("output").map_err(db::sql_error)?,
        ));
    }
    Ok(facts)
}

fn dependency_input(
    ids: &[String],
    facts: &[(String, Option<String>)],
    include_states: bool,
) -> Result<String, String> {
    let values = facts
        .iter()
        .map(|(_, output)| {
            output
                .as_deref()
                .map(serde_json::from_str)
                .transpose()
                .map_err(db::json_error)
                .map(|value| value.unwrap_or(Value::Null))
        })
        .collect::<Result<Vec<Value>, String>>()?;
    let value = if include_states {
        Value::Array(
            ids.iter()
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
        values.into_iter().next().unwrap_or(Value::Null)
    } else {
        Value::Object(ids.iter().cloned().zip(values).collect())
    };
    serde_json::to_string(&value).map_err(db::json_error)
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
    sqlx::query("UPDATE spaces_runs SET state=$1,state_version=state_version+1,wait_reason=NULL,updated_at=$2,completed_at=$2 WHERE id=$3 AND completed_at IS NULL")
        .bind(enum_text(state)?).bind(now).bind(run_id).execute(&mut **tx).await.map_err(db::sql_error)?;
    Ok(())
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

fn enum_text(value: impl serde::Serialize) -> Result<String, String> {
    serde_json::to_value(value)
        .map_err(db::json_error)?
        .as_str()
        .map(str::to_owned)
        .ok_or("Invalid enum".into())
}
