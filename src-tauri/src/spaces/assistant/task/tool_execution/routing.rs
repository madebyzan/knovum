use serde_json::Value;

use crate::spaces::{actions::SpaceActionDefinition, assistant::execution::gateway::Gateway, db};

pub(super) async fn selection(
    host: &Gateway,
    action: &SpaceActionDefinition,
    invocation_id: &str,
) -> Result<Option<Value>, String> {
    let Some(purpose) = action.routing_purpose else {
        return Ok(None);
    };
    let previous: Option<String> = sqlx::query_scalar(
        "SELECT target_selection FROM spaces_assistant_tool_operations WHERE invocation_id=$1",
    )
    .bind(invocation_id)
    .fetch_optional(&host.pool)
    .await
    .map_err(db::sql_error)?
    .flatten();
    if let Some(value) = previous {
        return serde_json::from_str(&value)
            .map(Some)
            .map_err(db::json_error);
    }
    let target = crate::ai::settings::default_selection(&host.app, purpose)
        .map_err(|error| error.to_string())?;
    crate::ai::settings::resolve_task(&host.app, target.clone(), purpose.execution_capability())
        .map_err(|error| error.to_string())?;
    serde_json::to_value(target)
        .map(Some)
        .map_err(db::json_error)
}
