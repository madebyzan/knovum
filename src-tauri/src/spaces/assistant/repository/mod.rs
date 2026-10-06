mod context;
mod conversations;
mod message_branches;
mod message_contract;
mod messages;
mod tasks;

pub use context::build as build_context;
pub(in crate::spaces::assistant) use context::load_history;
pub use conversations::{
    archive, delete, ensure_branch_idle, ensure_conversation, list_conversations, rename, unarchive,
};
pub use message_branches::{activate as activate_branch, branch, list as list_branches};
pub use message_contract::{
    AssistantMessage, AssistantMessageAttachment, AssistantMessageContent, AssistantMessageOutput,
    AssistantMessageRole,
};
pub use messages::{all_messages, append_message_in_transaction, list_messages};
pub(in crate::spaces::assistant) use tasks::{
    complete_tool_call, insert_task, recoverable_task, recoverable_tasks, start_tool_call,
    AssistantToolCallCompletion, AssistantToolCallStart, RecoverableTask,
};

#[cfg(test)]
mod tests;
