use sqlx::{Row, SqlitePool};

use crate::spaces::db;

pub(super) struct SourceRun {
    pub space_id: String,
    pub principal_kind: String,
    pub principal_id: String,
    pub kind: String,
    pub policy: String,
    pub input: String,
    pub budget: String,
}

pub(super) async fn load_source(pool: &SqlitePool, run_id: &str) -> Result<SourceRun, String> {
    let row = sqlx::query(
        "SELECT home_space_id,principal_kind,principal_id,kind,policy_snapshot,input_snapshot,budget,
         completed_at FROM spaces_runs WHERE id=$1",
    ).bind(run_id).fetch_optional(pool).await.map_err(db::sql_error)?
        .ok_or("run-unavailable: source Run is unavailable")?;
    if row
        .try_get::<Option<i64>, _>("completed_at")
        .map_err(db::sql_error)?
        .is_none()
    {
        return Err("run-retry-invalid: only a terminal Run can be retried".into());
    }
    let kind: String = row.try_get("kind").map_err(db::sql_error)?;
    if matches!(kind.as_str(), "assistant" | "external_agent") {
        return Err(
            "run-retry-unsupported: conversations must create an explicit new turn or branch"
                .into(),
        );
    }
    Ok(SourceRun {
        space_id: row.try_get("home_space_id").map_err(db::sql_error)?,
        principal_kind: row.try_get("principal_kind").map_err(db::sql_error)?,
        principal_id: row.try_get("principal_id").map_err(db::sql_error)?,
        kind,
        policy: row.try_get("policy_snapshot").map_err(db::sql_error)?,
        input: row.try_get("input_snapshot").map_err(db::sql_error)?,
        budget: row.try_get("budget").map_err(db::sql_error)?,
    })
}

pub(super) struct SourceStep {
    pub id: String,
    pub node_id: String,
    pub instance_path: String,
    pub node_kind: String,
    pub depends_on: String,
    pub configuration: String,
    pub state: String,
    pub input: String,
    pub output: Option<String>,
}

pub(super) async fn load_steps(pool: &SqlitePool, run_id: &str) -> Result<Vec<SourceStep>, String> {
    let rows = sqlx::query(
        "SELECT id,definition_node_id,instance_path,node_kind,depends_on,configuration,state,
         resolved_input,output FROM spaces_run_steps WHERE run_id=$1 ORDER BY created_at,id",
    )
    .bind(run_id)
    .fetch_all(pool)
    .await
    .map_err(db::sql_error)?;
    rows.iter()
        .map(|row| {
            Ok(SourceStep {
                id: row.try_get("id").map_err(db::sql_error)?,
                node_id: row.try_get("definition_node_id").map_err(db::sql_error)?,
                instance_path: row.try_get("instance_path").map_err(db::sql_error)?,
                node_kind: row.try_get("node_kind").map_err(db::sql_error)?,
                depends_on: row.try_get("depends_on").map_err(db::sql_error)?,
                configuration: row.try_get("configuration").map_err(db::sql_error)?,
                state: row.try_get("state").map_err(db::sql_error)?,
                input: row.try_get("resolved_input").map_err(db::sql_error)?,
                output: row.try_get("output").map_err(db::sql_error)?,
            })
        })
        .collect()
}
