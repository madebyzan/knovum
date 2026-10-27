use std::sync::Mutex;

use futures::future::BoxFuture;
use serde::de::DeserializeOwned;
use serde_json::Value;
use sqlx::SqlitePool;
use tauri::ipc::Channel;

use super::{ledger, projection, session::Session};
use crate::{
    agent_runtime::{
        process::Host,
        protocol::{
            control::AgentExecutionIdentity,
            messages::{AgentContentDelta, AgentProjection},
            models::{AgentModelAdmission, AgentModelCompletion, AgentModelFailure},
            tools::{AgentToolAdmission, AgentToolInvocation},
        },
    },
    ai::models::ModelExecutionTarget,
    spaces::{
        conversations::{
            builtin_agent::identity::TaskIds, tools::ToolCatalog, ConversationTaskUpdate,
            StartBuiltinAgentTaskRequest,
        },
        db,
    },
};

pub(in crate::spaces::conversations) struct Gateway {
    pub(in crate::spaces::conversations) app: tauri::AppHandle,
    pub(in crate::spaces::conversations) pool: SqlitePool,
    pub(in crate::spaces::conversations) target: ModelExecutionTarget,
    pub(in crate::spaces::conversations) profile: crate::ai::models::profile::ModelProfile,
    pub(in crate::spaces::conversations) request: StartBuiltinAgentTaskRequest,
    pub(in crate::spaces::conversations) ids: TaskIds,
    pub(in crate::spaces::conversations) catalog: ToolCatalog,
    pub(in crate::spaces::conversations) extensions:
        crate::extensions::contracts::AgentExtensionSnapshot,
    pub(in crate::spaces::conversations) session: Session,
    pub(in crate::spaces::conversations) updates: Option<Channel<ConversationTaskUpdate>>,
    pub(in crate::spaces::conversations) last_message: Mutex<Option<String>>,
}

impl Host for Gateway {
    fn dispatch(&self, method: String, params: Value) -> BoxFuture<'_, Result<Value, String>> {
        Box::pin(async move {
            match method.as_str() {
                "agentRuntime.requireRecovery" => {
                    let identity: AgentExecutionIdentity = decode(params)?;
                    self.require_identity(&identity).await?;
                    ledger::require_reconciled(&self.pool, &self.ids.task_id).await?;
                    Ok(Value::Null)
                }
                "agentRuntime.admitModel" => {
                    let request: AgentModelAdmission = decode(params)?;
                    self.require_identity(&request.identity).await?;
                    ledger::admit(&self.pool, &self.ids.task_id, request).await
                }
                "agentRuntime.completeModel" => {
                    let request: AgentModelCompletion = decode(params)?;
                    self.require_process_identity(&request.identity)?;
                    ledger::complete(&self.pool, &self.ids.task_id, request).await
                }
                "agentRuntime.failModel" => {
                    let request: AgentModelFailure = decode(params)?;
                    self.require_process_identity(&request.identity)?;
                    ledger::fail(&self.pool, &self.ids.task_id, request).await
                }
                "agentRuntime.admitTool" => self.admit_tool(decode(params)?).await,
                "agentRuntime.invokeTool" => {
                    let request: AgentToolInvocation = decode(params)?;
                    self.require_identity(&request.identity).await?;
                    self.announce_tool(&request.tool_call_id);
                    crate::spaces::conversations::builtin_agent::tool_execution::invoke(
                        self, request,
                    )
                    .await
                }
                "agentRuntime.commitProjection" => {
                    let request: AgentProjection = decode(params)?;
                    self.require_identity(&self.session.identity(&self.ids.run_id))
                        .await?;
                    projection::commit(&self.pool, &self.ids, request).await
                }
                _ => Err("builtin-agent-host-method-not-found".into()),
            }
        })
    }

    fn notify(&self, method: &str, params: Value) -> Result<(), String> {
        match method {
            "agentRuntime.contentDelta" => self.delta(decode(params)?),
            _ => Err("builtin-agent-host-method-not-found".into()),
        }
    }
}

impl Gateway {
    pub(in crate::spaces::conversations) async fn require_identity(
        &self,
        identity: &AgentExecutionIdentity,
    ) -> Result<(), String> {
        self.require_process_identity(identity)?;
        let epoch: i64 = sqlx::query_scalar("SELECT control_epoch FROM spaces_builtin_agent_tasks WHERE id=$1 AND run_id=$2 AND segment_id=$3")
            .bind(&self.ids.task_id).bind(&identity.run_id).bind(&identity.thread_id).fetch_one(&self.pool).await.map_err(db::sql_error)?;
        if u64::try_from(epoch).map_err(|_| "builtin-agent-control-epoch-invalid")?
            != identity.control_epoch
        {
            return Err("builtin-agent-execution-superseded".into());
        }
        Ok(())
    }

    fn require_process_identity(&self, identity: &AgentExecutionIdentity) -> Result<(), String> {
        let expected = self.session.identity(&self.ids.run_id);
        if identity.run_id != expected.run_id
            || identity.thread_id != expected.thread_id
            || identity.execution_revision != expected.execution_revision
            || identity.control_epoch != expected.control_epoch
        {
            return Err("builtin-agent-execution-identity-mismatch".into());
        }
        Ok(())
    }

    async fn admit_tool(&self, request: AgentToolAdmission) -> Result<Value, String> {
        self.require_identity(&request.identity).await?;
        crate::extensions::snapshots::require_current(&self.app, &self.extensions).await?;
        require_tool(&self.catalog, &request.name, &request.graph_namespace)?;
        if request.sibling_tools.iter().any(|name| name == "task")
            && self
                .catalog
                .binding_by_wire_name
                .get(&request.name)
                .is_some_and(|definition| !definition.read_only)
        {
            return Err("builtin-agent-research-write-conflict: complete research before publishing business changes".into());
        }
        let key = tool_key(
            &request.identity,
            &request.graph_namespace,
            &request.message_id,
            &request.tool_call_id,
            &request.name,
        );
        ledger::admit_tool(
            &self.pool,
            &self.ids.task_id,
            &self.ids.run_id,
            &key,
            &request.name,
        )
        .await
    }

    /// Marks where in the streamed answer a tool was called. Deltas and invocations arrive over
    /// the one runtime connection in the order the Agent produced them, so the mark lands
    /// between the words that led to the call and the words that follow it.
    fn announce_tool(&self, tool_call_id: &str) {
        if let Some(channel) = &self.updates {
            let _ = channel.send(ConversationTaskUpdate::ToolStarted {
                tool_call_id: tool_call_id.to_owned(),
            });
        }
    }

    /// Shows a chunk of the answer as it arrives. Only the process identity is checked: the
    /// chunk is not durable — the projection that stores the answer checks the control epoch
    /// against the database before it writes — and a database read here would cost the
    /// stream its order.
    fn delta(&self, request: AgentContentDelta) -> Result<(), String> {
        self.require_process_identity(&request.identity)?;
        let mut last = self
            .last_message
            .lock()
            .map_err(|_| "builtin-agent-stream-state-unavailable")?;
        if let Some(channel) = &self.updates {
            if last.as_ref().is_some_and(|id| id != &request.message_id) {
                let _ = channel.send(ConversationTaskUpdate::ContentBoundary);
            }
            let _ = channel.send(ConversationTaskUpdate::ContentDelta {
                task_id: self.ids.task_id.clone(),
                text: request.text,
            });
        }
        *last = Some(request.message_id);
        Ok(())
    }
}

pub(in crate::spaces::conversations) fn tool_key(
    identity: &AgentExecutionIdentity,
    namespace: &str,
    message_id: &str,
    call_id: &str,
    name: &str,
) -> String {
    use sha2::{Digest, Sha256};
    // Action wire names include their version; process/epoch/attempt identities
    // deliberately do not participate in the durable operation identity.
    let bytes = serde_json::to_vec(&(
        identity.execution_revision.as_str(),
        namespace,
        message_id,
        call_id,
        name,
    ))
    .expect("owned tool identity serializes");
    format!("builtin-agent:{}", hex::encode(Sha256::digest(bytes)))
}

fn decode<T: DeserializeOwned>(value: Value) -> Result<T, String> {
    serde_json::from_value(value).map_err(|_| "builtin-agent-invalid-host-parameters".into())
}

pub(in crate::spaces::conversations) fn require_tool(
    catalog: &ToolCatalog,
    name: &str,
    namespace: &str,
) -> Result<(), String> {
    let research = namespace.contains('|');
    if let Some(definition) = catalog.binding_by_wire_name.get(name) {
        if research && !definition.read_only {
            return Err("builtin-agent-research-write-denied".into());
        }
        return Ok(());
    }
    let allowed = match name {
        "ls" | "read_file" | "glob" | "grep" | "compact_conversation" => true,
        "write_file" | "edit_file" | "write_todos" | "task" => !research,
        _ => false,
    };
    if allowed {
        Ok(())
    } else {
        Err("builtin-agent-tool-not-allowed".into())
    }
}
