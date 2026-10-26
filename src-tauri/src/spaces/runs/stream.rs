//! A streamed Agent session records its updates as Run events.
//!
//! An update arrives per streamed chunk, so a write transaction per update would spend one
//! commit on a few dozen bytes and rewrite every accumulating row from its start. A stream
//! instead records whatever has queued in one transaction, in arrival order: the cost of a
//! streamed answer stays proportional to the answer, not to the number of chunks it arrived in.

use std::future::Future;

use serde_json::Value;
use sqlx::{Sqlite, SqlitePool, Transaction};

use super::{append_event, RunEventKind};
use crate::external_agents::session::AgentSessionUpdate;
use crate::foundation::app_error::AppError;
use crate::spaces::db;

/// How many updates may queue before a producer is told it has outrun the recorder, and how
/// many a single transaction may carry.
const QUEUE_CAPACITY: usize = 128;

/// An update larger than this is a runaway Agent, not an answer.
const MAX_UPDATE_TEXT_BYTES: usize = 64 * 1024;

/// One streamed update, in the form it is stored and read back in.
pub(crate) struct Streamed {
    pub(crate) kind: RunEventKind,
    pub(crate) payload: Value,
}

impl Streamed {
    /// The durable record of a session update, or nothing for an update that only ever
    /// reaches the screen.
    fn of(update: AgentSessionUpdate) -> Option<Self> {
        let (kind, payload) = match update {
            AgentSessionUpdate::Thought(_) => return None,
            AgentSessionUpdate::Message(text) => (
                RunEventKind::ExternalAgentMessage,
                serde_json::json!({"text": text}),
            ),
            AgentSessionUpdate::ToolCall {
                id,
                title,
                status,
                context,
            } => (
                RunEventKind::ExternalAgentToolStarted,
                serde_json::json!({"toolCallId": id, "title": title, "status": status, "context": context}),
            ),
            AgentSessionUpdate::ToolUpdate {
                id,
                title,
                status,
                context,
            } => (
                RunEventKind::ExternalAgentToolUpdated,
                serde_json::json!({"toolCallId": id, "title": title, "status": status, "context": context}),
            ),
        };
        Some(Self { kind, payload })
    }

    fn text_bytes(&self) -> usize {
        self.payload.as_object().map_or(0, |fields| {
            fields
                .values()
                .filter_map(Value::as_str)
                .map(str::len)
                .sum()
        })
    }

    /// The text a message update carried.
    pub(crate) fn text(&self) -> Option<&str> {
        self.payload.get("text").and_then(Value::as_str)
    }
}

/// What a surface writes beside the Run events of a batch.
pub(crate) trait Sink: Send + Sync + 'static {
    /// Work that belongs in the same transaction as the batch it describes.
    fn settle(
        &self,
        _tx: &mut Transaction<'_, Sqlite>,
        _batch: &[Streamed],
    ) -> impl Future<Output = Result<(), String>> + Send {
        async { Ok(()) }
    }
}

/// A stream whose updates need nothing beyond the Run events themselves: what tests drive a
/// stream with when the transcript a production sink settles is not under test.
#[cfg(test)]
pub(crate) struct RunEventsOnly;

#[cfg(test)]
impl Sink for RunEventsOnly {}

enum Command {
    Record(Streamed),
    Flush(tokio::sync::oneshot::Sender<Result<(), String>>),
}

/// The producer end of a session's recorder.
#[derive(Clone)]
pub(crate) struct Stream {
    updates: tokio::sync::mpsc::Sender<Command>,
}

impl Stream {
    /// Hands an update to the recorder. Returns once it is queued, not once it is durable.
    pub(crate) fn record(&self, update: AgentSessionUpdate) -> Result<(), AppError> {
        let Some(streamed) = Streamed::of(update) else {
            return Ok(());
        };
        if streamed.text_bytes() > MAX_UPDATE_TEXT_BYTES {
            return Err(AppError::invalid(
                "External Agent update exceeds the size limit",
            ));
        }
        self.updates
            .try_send(Command::Record(streamed))
            .map_err(|error| match error {
                tokio::sync::mpsc::error::TrySendError::Full(_) => {
                    AppError::internal("External Agent updates exceeded the processing budget")
                }
                tokio::sync::mpsc::error::TrySendError::Closed(_) => {
                    AppError::internal("External Agent update stream closed")
                }
            })
    }

    /// Waits until everything queued before the call is durable, and reports the first
    /// failure the recorder met since the previous flush.
    pub(crate) async fn flush(&self) -> Result<(), String> {
        let (acknowledge, recorded) = tokio::sync::oneshot::channel();
        self.updates
            .send(Command::Flush(acknowledge))
            .await
            .map_err(|_| "Agent update stream has closed")?;
        recorded
            .await
            .map_err(|_| "Agent update stream stopped before flush".to_owned())?
    }
}

/// Opens a recorder for a Run's streamed updates. It lives until every producer is dropped.
pub(crate) fn open<S: Sink>(pool: SqlitePool, run_id: String, sink: S) -> Stream {
    let (updates, mut commands) = tokio::sync::mpsc::channel(QUEUE_CAPACITY);
    tauri::async_runtime::spawn(async move {
        let mut failure: Option<String> = None;
        let mut queued = Vec::with_capacity(QUEUE_CAPACITY);
        let mut batch = Vec::with_capacity(QUEUE_CAPACITY);
        while commands.recv_many(&mut queued, QUEUE_CAPACITY).await > 0 {
            for command in queued.drain(..) {
                match command {
                    Command::Record(streamed) => batch.push(streamed),
                    Command::Flush(acknowledge) => {
                        record_pending(&pool, &run_id, &sink, &mut batch, &mut failure).await;
                        let _ = acknowledge.send(failure.take().map_or(Ok(()), Err));
                    }
                }
            }
            record_pending(&pool, &run_id, &sink, &mut batch, &mut failure).await;
        }
    });
    Stream { updates }
}

async fn record_pending<S: Sink>(
    pool: &SqlitePool,
    run_id: &str,
    sink: &S,
    batch: &mut Vec<Streamed>,
    failure: &mut Option<String>,
) {
    if batch.is_empty() {
        return;
    }
    let recorded = record_batch(pool, run_id, sink, batch).await;
    batch.clear();
    match recorded {
        Ok(()) => super::announce_change(),
        Err(error) => {
            log::error!("External Agent update failed: {error}");
            if failure.is_none() {
                *failure = Some(error);
            }
        }
    }
}

async fn record_batch<S: Sink>(
    pool: &SqlitePool,
    run_id: &str,
    sink: &S,
    batch: &[Streamed],
) -> Result<(), String> {
    let now = crate::foundation::clock::unix_time_millis()?;
    let mut tx = db::begin_write(pool).await?;
    sink.settle(&mut tx, batch).await?;
    for streamed in batch {
        append_event(
            &mut tx,
            run_id,
            None,
            None,
            streamed.kind,
            &streamed.payload,
            now,
        )
        .await?;
    }
    tx.commit().await.map_err(db::sql_error)
}

#[cfg(test)]
#[path = "tests/stream.rs"]
mod tests;
