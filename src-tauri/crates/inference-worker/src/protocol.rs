#[cfg(test)]
#[path = "tests/protocol.rs"]
mod tests;
use std::io::{BufRead, Read, Write};
use std::sync::{Arc, Mutex};

use knovum_inference_contracts::runtime::MAX_MESSAGE_BYTES;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Request {
    pub jsonrpc: String,
    pub id: Option<String>,
    pub method: String,
    pub params: Value,
}

pub(crate) fn read_frame(
    reader: &mut impl BufRead,
    maximum: usize,
) -> Result<Option<Value>, String> {
    let mut bytes = Vec::new();
    reader
        .take((maximum + 1) as u64)
        .read_until(b'\n', &mut bytes)
        .map_err(|error| error.to_string())?;
    if bytes.is_empty() {
        return Ok(None);
    }
    if bytes.len() > maximum || !bytes.ends_with(b"\n") {
        return Err("invalid_input: protocol frame is oversized or incomplete".into());
    }
    serde_json::from_slice(&bytes)
        .map(Some)
        .map_err(|error| format!("invalid_input: {error}"))
}

#[derive(Clone)]
pub(crate) struct Output(Arc<Mutex<Box<dyn Write + Send>>>);

impl Output {
    pub(crate) fn new(writer: impl Write + Send + 'static) -> Self {
        Self(Arc::new(Mutex::new(Box::new(writer))))
    }

    pub(crate) fn notification(&self, method: &str, params: &impl Serialize) -> Result<(), String> {
        self.send(&json!({"jsonrpc":"2.0","method":method,"params":params}))
    }

    pub(crate) fn respond(
        &self,
        request_id: &str,
        response: &impl Serialize,
    ) -> Result<(), String> {
        self.send(&json!({"jsonrpc":"2.0", "id":request_id, "result":response}))
    }

    pub(crate) fn error(&self, request_id: Option<&str>, error: &str) -> Result<(), String> {
        let code = error.split_once(':').map_or(error, |(code, _)| code);
        self.send(&json!({"jsonrpc":"2.0", "id":request_id, "error":{
            "code":-32000, "message":error, "data":{"code":code}
        }}))
    }

    fn send(&self, value: &Value) -> Result<(), String> {
        let mut bytes = serde_json::to_vec(value).map_err(|error| error.to_string())?;
        if bytes.len() >= MAX_MESSAGE_BYTES {
            return Err("resource_exhausted: protocol response exceeds 1 MiB".into());
        }
        bytes.push(b'\n');
        let mut output = self
            .0
            .lock()
            .map_err(|_| "worker_crashed: protocol output lock poisoned")?;
        output
            .write_all(&bytes)
            .and_then(|()| output.flush())
            .map_err(|error| error.to_string())
    }
}
