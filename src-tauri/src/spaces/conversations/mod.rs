mod builtin_agent;
pub(crate) mod commands;
mod context;
mod contract;
mod execution;
mod extensions;
mod external_agent;
pub(crate) mod outputs;
pub(crate) mod queries;
mod repository;
pub(crate) mod tool_operations;
mod tools;
pub(crate) mod upload;

pub(crate) fn recover_tasks(app: tauri::AppHandle) {
    builtin_agent::recover(app);
}

pub(crate) async fn resume_task(app: tauri::AppHandle, run_id: &str) -> Result<(), String> {
    builtin_agent::resume(app, run_id).await
}

pub(crate) use contract::ConversationTaskFinishedState;
pub use contract::{
    Conversation, ConversationMode, ConversationTaskAccepted, ConversationTaskUpdate,
    StartBuiltinAgentTaskRequest, SubmitConversationTaskRequest,
};
pub(crate) use repository::ensure_branch_idle;
pub(crate) use repository::{
    ConversationMessage, ConversationMessageAttachment, ConversationMessageContent,
    ConversationMessageOutput,
};

pub(crate) use commands::cancel_task;
pub(crate) mod prompt_policy;
