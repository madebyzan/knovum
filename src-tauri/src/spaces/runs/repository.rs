use sqlx::{Sqlite, SqlitePool, Transaction};

use super::run_rows::{enum_json, optional_enum_json, parse_summary};
use super::{RunKind, RunState, RunSummary, WaitReason};
use crate::spaces::{authorization::Principal, db};

pub(crate) struct NewRun<'a> {
    pub id: &'a str,
    pub home_space_id: &'a str,
    pub parent_run_id: Option<&'a str>,
    pub principal: &'a Principal,
    pub kind: RunKind,
    pub policy_snapshot: &'a serde_json::Value,
    pub input_snapshot: &'a serde_json::Value,
    pub budget: &'a serde_json::Value,
}

pub(crate) async fn create_run_in_transaction(
    tx: &mut Transaction<'_, Sqlite>,
    run: &NewRun<'_>,
    now: i64,
) -> Result<(), String> {
    sqlx::query(
        "INSERT INTO spaces_runs(
          id,home_space_id,parent_run_id,principal_kind,principal_id,kind,state,state_version,
          execution_epoch,wait_reason,policy_snapshot,input_snapshot,budget,created_at,updated_at
        ) VALUES($1,$2,$3,$4,$5,$6,'queued',0,0,NULL,$7,$8,$9,$10,$10)",
    )
    .bind(run.id)
    .bind(run.home_space_id)
    .bind(run.parent_run_id)
    .bind(run.principal.kind.as_str())
    .bind(&run.principal.id)
    .bind(enum_json(run.kind)?)
    .bind(serde_json::to_string(run.policy_snapshot).map_err(db::json_error)?)
    .bind(serde_json::to_string(run.input_snapshot).map_err(db::json_error)?)
    .bind(serde_json::to_string(run.budget).map_err(db::json_error)?)
    .bind(now)
    .execute(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    super::append_event(
        tx,
        run.id,
        None,
        None,
        crate::spaces::runs::RunEventKind::RunQueued,
        &serde_json::json!({}),
        now,
    )
    .await
    .map(|_| ())
}

pub(crate) async fn load_run(pool: &SqlitePool, run_id: &str) -> Result<RunSummary, String> {
    let row = sqlx::query("SELECT * FROM spaces_runs WHERE id=$1")
        .bind(run_id)
        .fetch_optional(pool)
        .await
        .map_err(db::sql_error)?
        .ok_or("Run is unavailable")?;
    parse_summary(&row)
}

pub(crate) async fn list_runs(
    pool: &SqlitePool,
    space_id: &str,
) -> Result<Vec<RunSummary>, String> {
    let rows = sqlx::query(
        "SELECT * FROM spaces_runs WHERE home_space_id=$1 ORDER BY created_at DESC LIMIT 500",
    )
    .bind(space_id)
    .fetch_all(pool)
    .await
    .map_err(db::sql_error)?;
    rows.iter().map(parse_summary).collect()
}

pub(crate) async fn transition_run(
    pool: &SqlitePool,
    run_id: &str,
    expected_version: i64,
    state: RunState,
    wait_reason: Option<WaitReason>,
) -> Result<RunSummary, String> {
    let now = crate::clock::unix_time_millis()?;
    let completed_at = state.is_terminal().then_some(now);
    let result = sqlx::query("UPDATE spaces_runs SET state=$1,state_version=state_version+1,wait_reason=$2,updated_at=$3,completed_at=$4 WHERE id=$5 AND state_version=$6 AND completed_at IS NULL")
        .bind(enum_json(state)?).bind(optional_enum_json(wait_reason)?).bind(now).bind(completed_at)
        .bind(run_id).bind(expected_version).execute(pool).await.map_err(db::sql_error)?;
    db::require_one(result, "Run state changed concurrently")?;
    let run = load_run(pool, run_id).await?;
    super::signals::announce();
    Ok(run)
}

pub(crate) async fn request_control(
    pool: &SqlitePool,
    run_id: &str,
    control: &str,
) -> Result<(), String> {
    let now = crate::clock::unix_time_millis()?;
    let sql = match control {
        "cancel" => "UPDATE spaces_runs SET cancel_requested_at=$1,execution_epoch=execution_epoch+1,updated_at=$1 WHERE id=$2 AND completed_at IS NULL",
        "pause" => "UPDATE spaces_runs SET pause_requested_at=$1,updated_at=$1 WHERE id=$2 AND completed_at IS NULL",
        "resume" => "UPDATE spaces_runs SET pause_requested_at=NULL,state=CASE WHEN state='paused' THEN 'queued' ELSE state END,state_version=state_version+1,updated_at=$1 WHERE id=$2 AND completed_at IS NULL",
        _ => return Err("Unsupported Run control".into()),
    };
    let result = sqlx::query(sql)
        .bind(now)
        .bind(run_id)
        .execute(pool)
        .await
        .map_err(db::sql_error)?;
    db::require_one(result, "Run is unavailable or already complete")?;
    super::signals::announce();
    Ok(())
}

pub(crate) async fn durable_action_id(
    pool: &SqlitePool,
    run_id: &str,
) -> Result<Option<String>, String> {
    let encoded: Option<String> =
        sqlx::query_scalar("SELECT input_snapshot FROM spaces_runs WHERE id=$1 AND kind='action'")
            .bind(run_id)
            .fetch_optional(pool)
            .await
            .map_err(db::sql_error)?;
    encoded
        .map(|value| {
            serde_json::from_str::<serde_json::Value>(&value)
                .map_err(db::json_error)?
                .get("actionId")
                .and_then(serde_json::Value::as_str)
                .map(str::to_owned)
                .ok_or_else(|| "action-run-invalid: actionId is missing".into())
        })
        .transpose()
}

pub(crate) async fn active_delegate_ids(
    pool: &SqlitePool,
    run_id: &str,
) -> Result<Vec<String>, String> {
    sqlx::query_scalar(
        "SELECT id FROM spaces_run_steps WHERE run_id=$1 AND node_kind='delegate_task'
         AND state='running'",
    )
    .bind(run_id)
    .fetch_all(pool)
    .await
    .map_err(db::sql_error)
}
