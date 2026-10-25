mod inputs;
mod output;

use serde_json::{Map, Value};
use tauri::AppHandle;

use crate::foundation::app_error::AppError;
use crate::spaces::actions;

pub(super) async fn prepare(
    app: &AppHandle,
    action_id: &str,
    space_id: &str,
    parameters: &Value,
    snapshot: &Value,
    authority: &super::action_execution::ExecutionAuthority,
) -> Result<Value, AppError> {
    let definition = actions::require(action_id)?;
    let resolved =
        inputs::resolve(app, space_id, action_id, definition.min_inputs, snapshot).await?;
    let mut request = Map::new();
    request.insert("inputs".into(), Value::Array(resolved.items.clone()));
    if definition.routing_purpose.is_some() {
        let target = snapshot
            .get("targets")
            .and_then(|targets| targets.get(action_id))
            .ok_or("action-target-missing: select an execution target for this Action")?;
        let target: crate::ai::tasks::TaskTargetSelection =
            serde_json::from_value(target.clone()).map_err(crate::spaces::db::json_error)?;
        let capability = definition
            .execution_capability
            .ok_or("action-target-invalid: Action has no execution capability")?;
        crate::ai::settings::resolve_task(app, target.clone(), capability)
            .map_err(|error| format!("action-target-unavailable: {error}"))?;
        request.insert(
            "target".into(),
            serde_json::to_value(target).map_err(crate::spaces::db::json_error)?,
        );
    }
    let frame = snapshot
        .get("outputFrame")
        .cloned()
        .map(serde_json::from_value)
        .transpose()
        .map_err(crate::spaces::db::json_error)?;
    output::reserve(
        app,
        space_id,
        action_id,
        parameters,
        &resolved,
        frame,
        &mut request,
        authority,
    )
    .await?;
    Ok(Value::Object(request))
}
