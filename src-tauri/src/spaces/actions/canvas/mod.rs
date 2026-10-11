use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::SqlitePool;

use super::{
    ActionInvocationOutcome, ActionInvocationRequest, SpaceActionExecutionMode, SpaceActionImpact,
};
use crate::ai::tasks::TaskTargetRef;
use crate::foundation::app_error::AppError;
use crate::spaces::{
    authorization::{create_grant, local_user_grant, PrincipalKind, RunScope},
    db,
    runs::{load_run, RunObservation},
};

#[derive(Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub(crate) struct StartSpaceActionRequest {
    pub request_id: String,
    pub space_id: String,
    pub action_id: String,
    #[ts(type = "Record<string, unknown>")]
    pub parameters: Value,
    pub sources: Vec<SpaceActionSource>,
    pub target: Option<TaskTargetRef>,
}

#[derive(Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub(crate) struct SpaceActionSource {
    pub node_id: String,
    pub knowledge_revision: String,
}

#[tauri::command]
pub(crate) async fn start_space_action(
    app: tauri::AppHandle,
    request: StartSpaceActionRequest,
) -> Result<RunObservation, AppError> {
    let definition = super::require(&request.action_id)?;
    let mut invocation = invocation(&request, definition)?;
    super::resolve_target(&app, &mut invocation)?;
    accept(db::pool(&app).await?, &request, invocation).await
}

fn invocation(
    request: &StartSpaceActionRequest,
    definition: &super::SpaceActionDefinition,
) -> Result<ActionInvocationRequest, AppError> {
    uuid::Uuid::parse_str(&request.request_id)
        .map_err(|_| "action-input-invalid: request ID must be a UUID")?;
    if !matches!(definition.execution_mode, SpaceActionExecutionMode::Durable)
        || definition.impact != SpaceActionImpact::CreatesResource
    {
        return Err(
            "action-input-invalid: canvas execution requires a resource-creating Action".into(),
        );
    }
    if definition.routing_purpose.is_none() && request.target.is_some() {
        return Err("action-target-invalid: local Action does not accept an AI target".into());
    }
    let mut targets = serde_json::Map::new();
    if let Some(target) = &request.target {
        targets.insert(
            request.action_id.clone(),
            serde_json::to_value(target.clone().with_temperature(0.2)).map_err(db::json_error)?,
        );
    }
    Ok(ActionInvocationRequest {
        invocation_id: request.request_id.clone(),
        idempotency_key: request.request_id.clone(),
        grant_id: request.request_id.clone(),
        space_id: request.space_id.clone(),
        action_id: request.action_id.clone(),
        action_version: definition.version,
        parameters: request.parameters.clone(),
        input_snapshot: json!({"targets": targets}),
        expected_revisions: Value::Null,
        parent_run_id: None,
    })
}

async fn accept(
    pool: &SqlitePool,
    request: &StartSpaceActionRequest,
    mut invocation: ActionInvocationRequest,
) -> Result<RunObservation, AppError> {
    let mut tx = db::begin_write(pool).await?;
    invocation.input_snapshot["orderedInputs"] =
        super::canvas_inputs::pin(&mut tx, &request.space_id, &request.sources).await?;
    let exists: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM spaces_authorization_grants WHERE id=$1")
            .bind(&request.request_id)
            .fetch_one(&mut *tx)
            .await
            .map_err(db::sql_error)?;
    if exists != 0 {
        let grant = crate::spaces::authorization::load_grant(&mut tx, &request.request_id).await?;
        let expected_scope = RunScope::Space {
            space_id: request.space_id.clone(),
        };
        if grant.principal.kind != PrincipalKind::User
            || grant.principal.id != "local-user"
            || grant.scope != expected_scope
        {
            return Err(
                "permission-denied: canvas authorization belongs to another principal".into(),
            );
        }
    } else {
        let now = crate::foundation::clock::unix_time_millis()?;
        let grant = local_user_grant(
            request.request_id.clone(),
            RunScope::Space {
                space_id: request.space_id.clone(),
            },
            vec![request.action_id.clone()],
            now,
        )?;
        create_grant(&mut tx, &grant, now).await?;
    }
    let ActionInvocationOutcome::AcceptedRun { run_id } =
        super::invoke_in_transaction(&mut tx, &invocation).await?
    else {
        return Err("action-executor-invalid: resource Action did not create a Run".into());
    };
    tx.commit().await.map_err(db::sql_error)?;
    crate::spaces::runs::announce_change();
    Ok(load_run(pool, &run_id).await?)
}

#[cfg(test)]
mod tests;
