use std::io;

use serde::{de::DeserializeOwned, Serialize};
use serde_json::{json, Value};
use tokio::io::{AsyncBufRead, AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::process::{ChildStdin, ChildStdout};

use crate::ai::contracts::runtime::MAX_MESSAGE_BYTES;

/// A worker rejection reason is a diagnostic for logs and dialogs, not a payload.
const MAX_REASON_BYTES: usize = 512;

pub(super) struct Protocol {
    input: ChildStdin,
    output: BufReader<ChildStdout>,
}

impl Protocol {
    pub(super) fn new(input: ChildStdin, output: ChildStdout) -> Self {
        Self {
            input,
            output: BufReader::new(output),
        }
    }

    pub(super) async fn request<P: Serialize, R: DeserializeOwned>(
        &mut self,
        request_id: &str,
        method: &str,
        params: &P,
    ) -> Result<R, String> {
        self.send_request(request_id, method, params).await?;
        self.read_response(request_id).await
    }

    pub(super) async fn send_request<P: Serialize>(
        &mut self,
        request_id: &str,
        method: &str,
        params: &P,
    ) -> Result<(), String> {
        self.send(&json!({"jsonrpc":"2.0","id":request_id,"method":method,"params":params}))
            .await
    }

    pub(super) async fn read_response<R: DeserializeOwned>(
        &mut self,
        request_id: &str,
    ) -> Result<R, String> {
        self.read_response_with_events(request_id, |_, _| {
            Err("invalid_output: unexpected worker notification".into())
        })
        .await
    }

    pub(super) async fn read_response_with_events<R: DeserializeOwned>(
        &mut self,
        request_id: &str,
        mut on_event: impl FnMut(&str, Value) -> Result<(), String>,
    ) -> Result<R, String> {
        loop {
            let response = self.read().await?;
            if let Some(method) = response.get("method").and_then(Value::as_str) {
                if response.get("jsonrpc").and_then(Value::as_str) != Some("2.0")
                    || response.get("id").is_some()
                {
                    return Err("invalid_output: worker notification envelope is invalid".into());
                }
                let params = response
                    .get("params")
                    .cloned()
                    .ok_or("invalid_output: worker notification has no parameters")?;
                on_event(method, params)?;
                continue;
            }
            return Self::decode_response(response, request_id);
        }
    }

    pub(super) fn decode_response<R: DeserializeOwned>(
        response: Value,
        request_id: &str,
    ) -> Result<R, String> {
        if response.get("jsonrpc").and_then(Value::as_str) != Some("2.0")
            || response.get("id").and_then(Value::as_str) != Some(request_id)
        {
            return Err("invalid_output: worker returned a mismatched JSON-RPC response".into());
        }
        match (response.get("result"), response.get("error")) {
            (Some(response_payload), None) => serde_json::from_value(response_payload.clone())
                .map_err(|error| format!("invalid_output: {error}")),
            (None, Some(error)) => {
                // The worker reports the full reason as the message and its leading code as
                // data.code. Keep the reason when it refines that code: the code alone cannot tell
                // apart the rejections that share it.
                let code = error
                    .get("data")
                    .and_then(|data| data.get("code"))
                    .and_then(Value::as_str)
                    .unwrap_or("invalid_output");
                let reason = error
                    .get("message")
                    .and_then(Value::as_str)
                    .filter(|reason| reason.len() <= MAX_REASON_BYTES && reason.starts_with(code));
                Err(reason.unwrap_or(code).to_owned())
            }
            _ => Err("invalid_output: worker returned an invalid JSON-RPC envelope".into()),
        }
    }

    pub(super) async fn notify<P: Serialize>(
        &mut self,
        method: &str,
        params: &P,
    ) -> Result<(), String> {
        self.send(&json!({"jsonrpc":"2.0","method":method,"params":params}))
            .await
    }

    async fn send(&mut self, value: &Value) -> Result<(), String> {
        let mut bytes = serde_json::to_vec(value).map_err(|error| error.to_string())?;
        if bytes.len() >= MAX_MESSAGE_BYTES {
            return Err("resource_exhausted: protocol message exceeds 1 MiB".into());
        }
        bytes.push(b'\n');
        self.input
            .write_all(&bytes)
            .await
            .map_err(|error| format!("worker_crashed: {error}"))
    }

    pub(super) async fn read(&mut self) -> Result<Value, String> {
        read_frame(&mut self.output).await
    }
}

async fn read_frame(reader: &mut (impl AsyncBufRead + Unpin)) -> Result<Value, String> {
    let mut bytes = Vec::new();
    reader
        .take((MAX_MESSAGE_BYTES + 1) as u64)
        .read_until(b'\n', &mut bytes)
        .await
        .map_err(|error| format!("worker_crashed: {error}"))?;
    if bytes.is_empty() {
        return Err("worker_crashed: worker closed its protocol stream".into());
    }
    if bytes.len() > MAX_MESSAGE_BYTES {
        return Err("invalid_output: worker protocol message exceeds 1 MiB".into());
    }
    if !bytes.ends_with(b"\n") {
        return Err("worker_crashed: worker closed its protocol stream".into());
    }
    serde_json::from_slice(&bytes).map_err(|error| format!("invalid_output: {error}"))
}

pub(super) fn io_error(error: io::Error) -> String {
    format!("worker_crashed: {error}")
}

#[cfg(test)]
#[path = "tests/protocol.rs"]
mod tests;
