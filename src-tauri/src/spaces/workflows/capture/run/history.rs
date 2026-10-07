use serde::Serialize;
use sqlx::{Row, SqlitePool};

use crate::spaces::{
    db,
    runs::{self, RunObservation},
};

#[derive(Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub(crate) struct CapturedWorkflowRun {
    pub run: RunObservation,
    pub workflow_id: String,
}

pub(crate) async fn list(
    pool: &SqlitePool,
    space_id: &str,
) -> Result<Vec<CapturedWorkflowRun>, String> {
    let rows = sqlx::query(
        "SELECT id, json_extract(policy_snapshot,'$.workflowId') AS workflow_id
         FROM spaces_runs WHERE kind='workflow' AND home_space_id=$1
         AND json_extract(policy_snapshot,'$.captureSchemaVersion')=1
         AND json_type(policy_snapshot,'$.workflowDefinition')='object'
         ORDER BY created_at DESC LIMIT 30",
    )
    .bind(space_id)
    .fetch_all(pool)
    .await
    .map_err(db::sql_error)?;
    let mut history = Vec::with_capacity(rows.len());
    for row in rows {
        let id: String = row.try_get("id").map_err(db::sql_error)?;
        history.push(CapturedWorkflowRun {
            run: runs::load_run(pool, &id).await?,
            workflow_id: row.try_get("workflow_id").map_err(db::sql_error)?,
        });
    }
    Ok(history)
}
