use std::{collections::HashMap, sync::Arc};

use futures::future::BoxFuture;
use serde::{de::DeserializeOwned, Serialize};
use serde_json::{json, Value};
use tokio::{
    process::{ChildStdin, ChildStdout},
    sync::{oneshot, Mutex},
    task::JoinSet,
};

use super::framing;

type Response = Result<Value, String>;

pub(crate) trait Host: Send + Sync {
    fn dispatch(&self, method: String, params: Value) -> BoxFuture<'_, Response>;
}

#[derive(Clone)]
pub(crate) struct Peer {
    input: Arc<Mutex<ChildStdin>>,
    pending: Arc<std::sync::Mutex<HashMap<String, oneshot::Sender<Response>>>>,
}

impl Peer {
    pub(super) fn new(input: ChildStdin) -> Self {
        Self {
            input: Arc::new(Mutex::new(input)),
            pending: Arc::new(std::sync::Mutex::new(HashMap::new())),
        }
    }

    pub(crate) async fn request<T: DeserializeOwned>(
        &self,
        method: &str,
        params: &impl Serialize,
    ) -> Result<T, String> {
        let id = uuid::Uuid::new_v4().to_string();
        let (sender, receiver) = oneshot::channel();
        {
            let mut pending = self
                .pending
                .lock()
                .map_err(|_| "assistant-protocol-poisoned")?;
            if pending.len() >= 64 {
                return Err("assistant-request-limit".into());
            }
            pending.insert(id.clone(), sender);
        }
        let outcome = async {
            self.send(json!({"jsonrpc":"2.0","id":id,"method":method,"params":params}))
                .await?;
            let value = receiver
                .await
                .map_err(|_| "assistant-process-disconnected")??;
            serde_json::from_value(value).map_err(|_| "assistant-invalid-response".to_owned())
        }
        .await;
        self.pending
            .lock()
            .map_err(|_| "assistant-protocol-poisoned")?
            .remove(&id);
        outcome
    }

    pub(crate) async fn notify(&self, method: &str, params: &impl Serialize) -> Result<(), String> {
        self.send(json!({"jsonrpc":"2.0","method":method,"params":params}))
            .await
    }

    async fn send(&self, value: Value) -> Result<(), String> {
        framing::write(&mut *self.input.lock().await, &value).await
    }

    pub(super) async fn listen(
        &self,
        output: ChildStdout,
        host: Arc<dyn Host>,
    ) -> Result<(), String> {
        let mut reader = framing::Frames::new(output);
        let mut handlers = JoinSet::new();
        let outcome = async {
        loop {
            tokio::select! {
                frame = reader.next() => {
                    let value = frame?;
                    if let Some(method) = value.get("method") {
                        let method = method.as_str().ok_or("assistant-invalid-method")?.to_owned();
                        let params = value.get("params").cloned().unwrap_or(Value::Null);
                        let id = value.get("id").map(|id| id.as_str().map(str::to_owned).ok_or("assistant-invalid-request-id")).transpose()?;
                        if handlers.len() >= 64 { return Err("assistant-request-limit".into()); }
                        let host = host.clone();
                        let peer = self.clone();
                        handlers.spawn(async move {
                            let outcome = host.dispatch(method, params).await;
                            if let Some(id) = id {
                                let response = match outcome {
                                    Ok(value) => json!({"jsonrpc":"2.0","id":id,"result":value}),
                                    Err(error) => json!({"jsonrpc":"2.0","id":id,"error":{"code":-32000,"message":error}}),
                                };
                                peer.send(response).await?;
                            } else { outcome?; }
                            Ok::<_, String>(())
                        });
                    } else { self.accept(value)?; }
                }
                outcome = handlers.join_next(), if !handlers.is_empty() => {
                    outcome.ok_or("assistant-handler-missing")?.map_err(|_| "assistant-host-handler-failed")??;
                }
            }
        }
        }.await;
        // Requests already received may contain paid model usage. Finish their
        // host transactions even when the Python pipe closes during cancellation.
        while let Some(completed) = handlers.join_next().await {
            if let Err(error) = completed {
                log::error!("Assistant host handler did not settle: {error}");
            }
        }
        outcome
    }

    fn accept(&self, value: Value) -> Result<(), String> {
        let id = value["id"]
            .as_str()
            .ok_or("assistant-invalid-response-id")?;
        let sender = self
            .pending
            .lock()
            .map_err(|_| "assistant-protocol-poisoned")?
            .remove(id)
            .ok_or("assistant-unexpected-response")?;
        let outcome = if let Some(error) = value.get("error") {
            let message = error["message"].as_str().ok_or("assistant-invalid-error")?;
            Err(message.to_owned())
        } else {
            Ok(value
                .get("result")
                .cloned()
                .ok_or("assistant-response-missing-result")?)
        };
        sender
            .send(outcome)
            .map_err(|_| "assistant-response-unobserved".into())
    }

    pub(super) fn disconnect(&self) {
        if let Ok(mut pending) = self.pending.lock() {
            for (_, sender) in pending.drain() {
                let _ = sender.send(Err("assistant-process-disconnected".into()));
            }
        }
    }
}
