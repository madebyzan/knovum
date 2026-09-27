mod agent_task;
pub(crate) mod commands;
mod contract;
pub(crate) mod queries;
mod repository;
mod task;
mod tools;
pub(crate) mod upload;

pub(crate) fn recover_tasks(app: tauri::AppHandle) {
    task::recover(app);
}

pub(crate) async fn resume_task(app: tauri::AppHandle, run_id: &str) -> Result<(), String> {
    task::resume(app, run_id).await
}

pub(crate) async fn require_paused_checkpoint(
    pool: &sqlx::SqlitePool,
    run_id: &str,
) -> Result<(), String> {
    task::require_paused_checkpoint(pool, run_id).await
}

pub(crate) use contract::AssistantPermissionOption;
pub use contract::{
    AssistantConversation, AssistantMode, AssistantTaskAccepted, AssistantTaskUpdate,
    StartAssistantTaskRequest, SubmitAssistantTaskRequest,
};
pub(crate) use repository::ensure_branch_idle;
pub(crate) use repository::{
    AssistantMessage, AssistantMessageAttachment, AssistantMessageContent,
};

pub(crate) use commands::cancel_task;
