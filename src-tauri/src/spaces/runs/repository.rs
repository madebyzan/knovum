use sqlx::{types::Json, Sqlite, SqlitePool, Transaction};

use super::run_rows::{enum_text, optional_enum_text, parse_summary};
use super::{RunKind, RunObservation, RunState, WaitReason};
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
    .bind(enum_text(run.kind)?)
    .bind(Json(run.policy_snapshot))
    .bind(Json(run.input_snapshot))
    .bind(Json(run.budget))
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

pub(crate) async fn load_run(pool: &SqlitePool, run_id: &str) -> Result<RunObservation, String> {
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
) -> Result<Vec<RunObservation>, String> {
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
) -> Result<RunObservation, String> {
    let now = crate::clock::unix_time_millis()?;
    let completed_at = state.is_terminal().then_some(now);
    let run_write = sqlx::query("UPDATE spaces_runs SET state=$1,state_version=state_version+1,wait_reason=$2,updated_at=$3,completed_at=$4 WHERE id=$5 AND state_version=$6 AND completed_at IS NULL")
        .bind(enum_text(state)?).bind(optional_enum_text(wait_reason)?).bind(now).bind(completed_at)
        .bind(run_id).bind(expected_version).execute(pool).await.map_err(db::sql_error)?;
    db::require_one(run_write, "Run state changed concurrently")?;
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
        "cancel" => "WITH RECURSIVE run_tree(id) AS (
            SELECT id FROM spaces_runs WHERE id=$2 AND completed_at IS NULL
            UNION ALL SELECT child.id FROM spaces_runs child JOIN run_tree parent ON child.parent_run_id=parent.id
            WHERE child.completed_at IS NULL)
            UPDATE spaces_runs SET cancel_requested_at=COALESCE(cancel_requested_at,$1),
            execution_epoch=execution_epoch+CASE WHEN cancel_requested_at IS NULL THEN 1 ELSE 0 END,
            updated_at=$1 WHERE id IN (SELECT id FROM run_tree)",
        "pause" => "UPDATE spaces_runs SET pause_requested_at=$1,updated_at=$1 WHERE id=$2 AND completed_at IS NULL",
        "resume" => "UPDATE spaces_runs SET pause_requested_at=NULL,state=CASE WHEN state='paused' THEN 'queued' ELSE state END,state_version=state_version+1,updated_at=$1 WHERE id=$2 AND completed_at IS NULL",
        _ => return Err("Unsupported Run control".into()),
    };
    let run_write = sqlx::query(sql)
        .bind(now)
        .bind(run_id)
        .execute(pool)
        .await
        .map_err(db::sql_error)?;
    if run_write.rows_affected() == 0 {
        return Err("Run is unavailable or already complete".into());
    }
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
