use serde_json::Value;
use tauri::AppHandle;

use super::shared::completed;
use crate::spaces::runs::{AttemptClaim, StepOutcome};

pub(super) async fn execute(
    app: &AppHandle,
    claim: &AttemptClaim,
    runtime_id: &str,
) -> Result<StepOutcome, String> {
    let kind = serde_json::from_value::<crate::ai::contracts::AgentKind>(Value::String(
        runtime_id.to_owned(),
    ))
    .map_err(|_| "delegate-runtime-unavailable: runtime ID is not registered")?;
    let prompt = claim
        .resolved_input
        .get("prompt")
        .and_then(Value::as_str)
        .map(str::to_owned)
        .unwrap_or_else(|| claim.resolved_input.to_string());
    let artifacts = crate::spaces::agent_artifacts::create_isolated_agent_artifacts(
        app.clone(),
        crate::ai::contracts::AgentCreationRequest {
            kind,
            prompt,
            attachments: Vec::new(),
            space_id: claim.home_space_id.clone(),
            session_id: claim.step_instance_id.clone(),
        },
        event_channel(app, claim),
    )
    .await
    .map_err(|error| error.to_string())?;
    completed(serde_json::to_value(artifacts).map_err(crate::spaces::db::json_error)?)
}

fn event_channel(
    app: &AppHandle,
    claim: &AttemptClaim,
) -> tauri::ipc::Channel<crate::ai::contracts::AgentCreationEvent> {
    let app = app.clone();
    let run_id = claim.run_id.clone();
    let step_id = claim.step_instance_id.clone();
    tauri::ipc::Channel::new(move |body| {
        let tauri::ipc::InvokeResponseBody::Json(json) = body else {
            return Ok(());
        };
        let event = serde_json::from_str(&json)?;
        if let crate::ai::contracts::AgentCreationEvent::PermissionRequested {
            request_id,
            title,
            options,
        } = event
        {
            let app = app.clone();
            let run_id = run_id.clone();
            let step_id = step_id.clone();
            tauri::async_runtime::spawn(async move {
                if let Ok(pool) = crate::spaces::db::pool(&app).await {
                    if let Ok(mut tx) = crate::spaces::db::begin_write(pool).await {
                        let now = crate::clock::unix_time_millis().unwrap_or_default();
                        let payload = serde_json::json!({
                            "sessionId": step_id, "requestId": request_id,
                            "title": title, "options": options,
                        });
                        let _ = crate::spaces::runs::append_event(
                            &mut tx,
                            &run_id,
                            Some(&step_id),
                            None,
                            crate::spaces::runs::RunEventKind::ExternalAgentPermissionRequested,
                            &payload,
                            now,
                        )
                        .await;
                        let _ = tx.commit().await;
                    }
                }
            });
        }
        Ok(())
    })
}
