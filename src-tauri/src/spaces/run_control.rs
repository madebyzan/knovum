use crate::{app_error::AppError, spaces::runs::RunKind};
use serde::Serialize;

#[derive(Clone, Debug, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub struct RunControlReceipt {
    pub interruption_errors: Vec<AppError>,
}

impl RunControlReceipt {
    fn accepted() -> Self {
        Self {
            interruption_errors: Vec::new(),
        }
    }

    fn interrupted(&mut self, code: &'static str, owner: &str, error: impl std::fmt::Display) {
        self.interruption_errors.push(AppError::new(
            code,
            format!("{owner} interruption remains pending: {error}"),
        ));
    }
}

#[tauri::command]
pub async fn cancel_run(
    app: tauri::AppHandle,
    run_id: String,
) -> Result<RunControlReceipt, AppError> {
    let pool = crate::spaces::db::pool(&app).await?;
    let run = crate::spaces::runs::load_run(pool, &run_id).await?;
    crate::spaces::runs::request_cancellation(pool, &run_id).await?;
    let mut receipt = RunControlReceipt::accepted();
    if let Err(error) = crate::spaces::runs::cancel_active_attempts(pool, &run_id).await {
        receipt.interrupted(
            "run_control_active_attempt_interrupt_failed",
            "Local Run attempt",
            error,
        );
    }
    if run.kind == RunKind::ExternalAgent {
        match crate::spaces::external_conversations::cancel_by_run(&app, &run_id).await {
            Ok(true) => {}
            Ok(false) => {
                if let Err(error) = crate::ai::executions::cancel(&run_id) {
                    receipt.interrupted("run_control_ai_interrupt_failed", "AI execution", error);
                }
            }
            Err(error) => receipt.interrupted(
                "run_control_external_agent_interrupt_failed",
                "External Agent execution",
                error,
            ),
        }
    }
    if run.kind == RunKind::BuiltinAgent {
        if let Err(error) = crate::spaces::conversations::cancel_task(&run_id) {
            receipt.interrupted(
                "run_control_builtin_agent_interrupt_failed",
                "Built-in Agent execution",
                error,
            );
        }
    }
    Ok(receipt)
}

#[tauri::command]
pub async fn pause_run(
    app: tauri::AppHandle,
    run_id: String,
) -> Result<RunControlReceipt, AppError> {
    let pool = crate::spaces::db::pool(&app).await?;
    let kind = crate::spaces::runs::load_run(pool, &run_id).await?.kind;
    if kind == RunKind::ExternalAgent {
        return Err(
            "run-control-unsupported: External Agent sessions support cancellation, not pause"
                .into(),
        );
    }
    crate::spaces::runs::request_pause(pool, &run_id).await?;
    let mut receipt = RunControlReceipt::accepted();
    if kind == RunKind::BuiltinAgent {
        if let Err(error) = crate::spaces::conversations::cancel_task(&run_id) {
            receipt.interrupted(
                "run_control_builtin_agent_interrupt_failed",
                "Built-in Agent execution",
                error,
            );
        }
    }
    Ok(receipt)
}

#[tauri::command]
pub async fn resume_run(app: tauri::AppHandle, run_id: String) -> Result<(), AppError> {
    let pool = crate::spaces::db::pool(&app).await?;
    let run = crate::spaces::runs::load_run(pool, &run_id).await?;
    if run.kind == RunKind::BuiltinAgent {
        return Ok(crate::spaces::conversations::resume_task(app, &run_id).await?);
    }
    crate::spaces::runs::request_resume(pool, &run_id).await?;
    Ok(())
}
