mod inputs;
mod output;

use serde_json::{Map, Value};
use tauri::AppHandle;

use crate::{
    ai::{contracts::RoutingPurpose, settings},
    spaces::actions,
};

pub(crate) async fn prepare(
    app: &AppHandle,
    action_id: &str,
    run_id: &str,
    space_id: &str,
    parameters: &Value,
    snapshot: &Value,
) -> Result<Value, String> {
    let definition = actions::require(action_id)?;
    let resolved =
        inputs::resolve(app, space_id, action_id, definition.input_roles, snapshot).await?;
    let mut request = Map::new();
    request.insert("inputs".into(), Value::Array(resolved.items.clone()));
    if let Some(primary) = resolved.items.first() {
        request.insert("primary".into(), primary.clone());
    }
    if let Some(purpose) = definition.routing_purpose {
        let target = find_target(snapshot, action_id)
            .map(serde_json::from_value)
            .transpose()
            .map_err(crate::spaces::db::json_error)?
            .unwrap_or(
                settings::default_selection(app, purpose_for(action_id, purpose))
                    .map_err(|error| error.to_string())?,
            );
        request.insert(
            "target".into(),
            serde_json::to_value(target).map_err(crate::spaces::db::json_error)?,
        );
    }
    output::reserve(
        app,
        space_id,
        run_id,
        action_id,
        parameters,
        &resolved,
        &mut request,
    )
    .await?;
    Ok(Value::Object(request))
}

fn find_target(value: &Value, action_id: &str) -> Option<Value> {
    match value {
        Value::Object(values) => {
            if values.get("type").and_then(Value::as_str) == Some("model") {
                return Some(value.clone());
            }
            if let Some(target) = values
                .get("targets")
                .and_then(|targets| targets.get(action_id))
            {
                return Some(target.clone());
            }
            values
                .values()
                .find_map(|value| find_target(value, action_id))
        }
        Value::Array(values) => values
            .iter()
            .find_map(|value| find_target(value, action_id)),
        _ => None,
    }
}

fn purpose_for(action_id: &str, fallback: RoutingPurpose) -> RoutingPurpose {
    match action_id {
        "draft-script" | "draft-social-post" => RoutingPurpose::Writing,
        _ => fallback,
    }
}
