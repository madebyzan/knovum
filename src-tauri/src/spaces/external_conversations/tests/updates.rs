use super::super::repository::AgentMessageWrite;
use super::{message_write, Streamed};
use crate::spaces::runs::RunEventKind;

fn message(text: &str) -> Streamed {
    Streamed {
        kind: RunEventKind::ExternalAgentMessage,
        payload: serde_json::json!({"text": text}),
    }
}

fn tool(kind: RunEventKind) -> Streamed {
    Streamed {
        kind,
        payload: serde_json::json!({"toolCallId": "call", "title": "Read"}),
    }
}

#[test]
fn commentary_in_a_batch_continues_the_assistant_message() {
    let batch = [message("Looking "), message("into it.")];
    assert!(matches!(
        message_write(&batch),
        Some(AgentMessageWrite::Append(text)) if text == "Looking into it."
    ));
}

#[test]
fn a_tool_call_in_a_batch_ends_the_commentary_before_it() {
    let batch = [
        message("Reading "),
        message("the file."),
        tool(RunEventKind::ExternalAgentToolStarted),
        message("Done."),
    ];
    assert!(matches!(
        message_write(&batch),
        Some(AgentMessageWrite::Replace(text)) if text == "Done."
    ));
}

#[test]
fn a_batch_that_ends_on_a_tool_call_leaves_no_commentary() {
    let batch = [
        message("Reading."),
        tool(RunEventKind::ExternalAgentToolStarted),
    ];
    assert!(matches!(
        message_write(&batch),
        Some(AgentMessageWrite::Replace(text)) if text.is_empty()
    ));
}

#[test]
fn a_batch_without_commentary_leaves_the_assistant_message_alone() {
    let batch = [tool(RunEventKind::ExternalAgentToolUpdated)];
    assert!(message_write(&batch).is_none());
}

#[tokio::test]
async fn an_accepted_flush_survives_session_registry_release() {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    crate::spaces::db::migrate(&pool).await.unwrap();
    let id = uuid::Uuid::new_v4().to_string();
    super::active_updates().lock().unwrap().insert(
        id.clone(),
        crate::spaces::runs::stream::open(
            pool,
            "run".to_owned(),
            crate::spaces::runs::stream::RunEventsOnly,
        ),
    );
    let flush = super::flush(&id);
    super::release(&id);
    flush.await.unwrap();
    assert!(super::flush(&id).await.is_err());
}
