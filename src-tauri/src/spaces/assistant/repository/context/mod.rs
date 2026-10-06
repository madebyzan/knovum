use sqlx::{Row, SqlitePool};

use crate::spaces::db;

const RECENT_MESSAGE_COUNT: usize = 24;
const MAX_CONTEXT_CHARACTERS: usize = 20_000;
const SUMMARY_CHARACTERS: usize = 4_000;

pub async fn build(
    pool: &SqlitePool,
    conversation_id: &str,
    branch_id: &str,
) -> Result<Option<String>, String> {
    let rows = sqlx::query(
        "SELECT id,role,content FROM spaces_assistant_messages
         WHERE conversation_id=$1 AND branch_id=$2 ORDER BY created_at DESC,rowid DESC LIMIT 200",
    )
    .bind(conversation_id)
    .bind(branch_id)
    .fetch_all(pool)
    .await
    .map_err(db::sql_error)?;
    if rows.is_empty() {
        return Ok(None);
    }
    let mut messages = rows.into_iter().map(parse).collect::<Result<Vec<_>, _>>()?;
    messages.reverse();
    let split = messages.len().saturating_sub(RECENT_MESSAGE_COUNT);
    let (older, recent) = messages.split_at(split);
    let summary = summarize(older);
    let mut transcript = String::new();
    if !summary.is_empty() {
        transcript.push_str("Earlier conversation summary:\n");
        transcript.push_str(&summary);
        transcript.push_str("\n\n");
    }
    transcript.push_str("Recent conversation:\n");
    let mut remaining = MAX_CONTEXT_CHARACTERS - transcript.chars().count();
    let mut turns = Vec::new();
    for message in recent.iter().rev() {
        let turn = format!("{}: {}\n", message.role, message.text);
        let bounded: String = turn.chars().take(remaining.min(4_000)).collect();
        remaining -= bounded.chars().count();
        turns.push(bounded);
        if remaining == 0 {
            break;
        }
    }
    for turn in turns.into_iter().rev() {
        transcript.push_str(&turn);
    }
    Ok((!transcript.is_empty()).then_some(transcript))
}

pub(in crate::spaces::assistant) async fn load_history(
    pool: &SqlitePool,
    conversation_id: &str,
    branch_id: &str,
) -> Result<Vec<crate::agent_runtime::protocol::messages::AssistantInputMessage>, String> {
    use crate::agent_runtime::protocol::messages::{
        AssistantContentPart, AssistantInputMessage, AssistantInputRole,
    };
    let rows = sqlx::query("SELECT id,role,content FROM spaces_assistant_messages WHERE conversation_id=$1 AND branch_id=$2 AND role IN ('user','assistant') ORDER BY created_at,rowid")
        .bind(conversation_id).bind(branch_id).fetch_all(pool).await.map_err(db::sql_error)?;
    rows.into_iter()
        .map(|row| {
            let message = parse(row)?;
            let role = match message.role.as_str() {
                "user" => AssistantInputRole::User,
                "assistant" => AssistantInputRole::Assistant,
                _ => unreachable!("query selects conversation roles"),
            };
            Ok(AssistantInputMessage {
                id: format!("history:{}", message.id),
                role,
                content: vec![AssistantContentPart::Text { text: message.text }],
            })
        })
        .collect()
}

struct ContextMessage {
    id: String,
    role: String,
    text: String,
}

fn parse(row: sqlx::sqlite::SqliteRow) -> Result<ContextMessage, String> {
    let encoded: String = row.try_get("content").map_err(db::sql_error)?;
    let content: super::AssistantMessageContent =
        serde_json::from_str(&encoded).map_err(db::json_error)?;
    let body = content
        .parts
        .iter()
        .filter_map(context_part)
        .collect::<Vec<_>>()
        .join(" ");
    Ok(ContextMessage {
        id: row.try_get("id").map_err(db::sql_error)?,
        role: row.try_get("role").map_err(db::sql_error)?,
        text: format!("{}{}", resource_references(content), body),
    })
}

fn resource_references(message: super::AssistantMessageContent) -> String {
    let outputs = message.outputs;
    let attachments = message
        .attachments
        .into_iter()
        .filter_map(|attachment| match attachment {
            super::AssistantMessageAttachment::Node {
                node_id,
                title,
                kind,
                ..
            } => Some(super::AssistantMessageOutput {
                node_id,
                title,
                kind,
            }),
            super::AssistantMessageAttachment::File { .. } => None,
        })
        .collect::<Vec<_>>();
    if outputs.is_empty() && attachments.is_empty() {
        return String::new();
    }
    format!(
        "Canvas resource references (read current nodes, not old file paths): {}\n",
        serde_json::json!({"outputs": outputs, "attachments": attachments})
    )
}

fn context_part(part: &crate::ai::contracts::ModelContentPart) -> Option<String> {
    match part {
        crate::ai::contracts::ModelContentPart::Text { text } => Some(text.clone()),
        crate::ai::contracts::ModelContentPart::ToolResult { name, .. } => {
            Some(format!("[tool {name} completed]"))
        }
        _ => None,
    }
}

fn summarize(messages: &[ContextMessage]) -> String {
    let mut summary = String::new();
    for message in messages.iter().rev() {
        let excerpt = message.text.chars().take(320).collect::<String>();
        append_bounded(&mut summary, &format!("{}: {}\n", message.role, excerpt));
    }
    summary
}

fn append_bounded(target: &mut String, value: &str) {
    let remaining = SUMMARY_CHARACTERS.saturating_sub(target.chars().count());
    target.extend(value.chars().take(remaining));
}

#[cfg(test)]
mod tests;
