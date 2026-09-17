use std::collections::HashSet;

use sqlx::{Sqlite, Transaction};

use super::contracts::{PackageWorkflowRunRow, RestorePackageRequest};
use crate::spaces::db;

pub async fn insert(
    tx: &mut Transaction<'_, Sqlite>,
    request: &RestorePackageRequest,
) -> Result<(), String> {
    for run in &request.creation_workflow_runs {
        validate(run)?;
        sqlx::query(
            "INSERT INTO spaces_creation_workflow_runs(id,space_id,workflow_id,workflow_version,status,error,created_at,updated_at,completed_at) \
             VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9)",
        )
        .bind(&run.id).bind(&request.space.id).bind(&run.workflow_id).bind(run.workflow_version)
        .bind(&run.status).bind(&run.error).bind(run.created_at).bind(run.updated_at)
        .bind(run.completed_at).execute(&mut **tx).await.map_err(db::sql_error)?;
        for step in &run.steps {
            let outputs = serde_json::to_string(&step.output_node_ids).map_err(db::json_error)?;
            sqlx::query(
                "INSERT INTO spaces_creation_workflow_steps(run_id,step_id,action,status,output_node_ids,error,created_at,updated_at,completed_at) \
                 VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9)",
            )
            .bind(&run.id).bind(&step.id).bind(&step.action).bind(&step.status).bind(outputs)
            .bind(&step.error).bind(step.created_at).bind(step.updated_at).bind(step.completed_at)
            .execute(&mut **tx).await.map_err(db::sql_error)?;
        }
    }
    Ok(())
}

fn validate(run: &PackageWorkflowRunRow) -> Result<(), String> {
    let run_state = (run.status == "completed" && run.error.is_empty())
        || (run.status == "failed" && !run.error.trim().is_empty())
        || (run.status == "cancelled" && run.error.is_empty());
    let labels = !run.id.trim().is_empty()
        && !run.workflow_id.trim().is_empty()
        && run.workflow_version > 0
        && !run.steps.is_empty();
    let mut step_ids = HashSet::new();
    let steps = run.steps.iter().all(|step| {
        let state = match step.status.as_str() {
            "succeeded" => step.completed_at.is_some() && step.error.is_empty(),
            "failed" => step.completed_at.is_some() && !step.error.trim().is_empty(),
            "cancelled" => step.completed_at.is_some() && step.error.is_empty(),
            "pending" => step.completed_at.is_none() && step.error.is_empty(),
            _ => false,
        };
        state
            && !step.id.trim().is_empty()
            && !step.action.trim().is_empty()
            && step_ids.insert(&step.id)
            && step.output_node_ids.iter().all(|id| !id.trim().is_empty())
    });
    let completion = match run.status.as_str() {
        "completed" => run.steps.iter().all(|step| step.status == "succeeded"),
        "failed" => run.steps.iter().any(|step| step.status == "failed"),
        "cancelled" => {
            run.steps.iter().any(|step| step.status == "cancelled")
                && run.steps.iter().all(|step| step.status != "failed")
        }
        _ => false,
    };
    if run_state && labels && steps && completion {
        Ok(())
    } else {
        Err("Package Creation Workflow run is invalid".into())
    }
}
