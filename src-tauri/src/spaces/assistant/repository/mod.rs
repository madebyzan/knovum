mod context;
mod conversations;
mod message_branches;
mod message_contract;
mod messages;
mod tasks;

pub use context::build as build_context;
pub use conversations::{
    archive, delete, ensure_branch_idle, ensure_conversation, list_conversations, rename, unarchive,
};
pub use message_branches::branch;
pub(crate) use message_contract::attachment_summaries;
pub use message_contract::{
    AssistantMessage, AssistantMessageAttachment, AssistantMessageContent, AssistantMessageRole,
};
pub use messages::{all_messages, append_message, append_message_in_transaction, list_messages};
pub use tasks::{
    complete_tool_call, complete_turn, insert_task, recoverable_task, recoverable_tasks,
    save_checkpoint, start_tool_call, start_turn, RecoverableTask,
};

#[cfg(test)]
mod tests;
