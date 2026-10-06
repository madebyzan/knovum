mod agent_task;
pub(crate) mod commands;
mod contract;
mod execution;
pub(crate) mod outputs;
pub(crate) mod queries;
mod repository;
mod skills;
mod task;
mod tools;
pub(crate) mod upload;

pub(crate) fn recover_tasks(app: tauri::AppHandle) {
    task::recover(app);
}

pub(crate) async fn resume_task(app: tauri::AppHandle, run_id: &str) -> Result<(), String> {
    task::resume(app, run_id).await
}

pub(crate) use contract::AssistantTaskFinishedState;
pub use contract::{
    AssistantConversation, AssistantMode, AssistantTaskAccepted, AssistantTaskUpdate,
    StartAssistantTaskRequest, SubmitAssistantTaskRequest,
};
pub(crate) use repository::ensure_branch_idle;
pub(crate) use repository::{
    AssistantMessage, AssistantMessageAttachment, AssistantMessageContent, AssistantMessageOutput,
};

pub(crate) use commands::cancel_task;
pub(crate) mod prompt_policy;
