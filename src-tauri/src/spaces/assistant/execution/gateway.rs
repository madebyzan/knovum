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
            control::AssistantExecutionIdentity,
            messages::{AssistantContentDelta, AssistantProjection},
            models::{AssistantModelAdmission, AssistantModelCompletion, AssistantModelFailure},
            tools::{AssistantToolAdmission, AssistantToolInvocation},
        },
    },
    ai::models::ModelExecutionTarget,
    spaces::{
        assistant::{
            task::identity::TaskIds, tools::ToolCatalog, AssistantMode, AssistantTaskUpdate,
            StartAssistantTaskRequest,
        },
        db,
    },
};

pub(in crate::spaces::assistant) struct Gateway {
    pub(in crate::spaces::assistant) app: tauri::AppHandle,
    pub(in crate::spaces::assistant) pool: SqlitePool,
    pub(in crate::spaces::assistant) target: ModelExecutionTarget,
    pub(in crate::spaces::assistant) profile:
        crate::ai::settings::contracts::AssistantModelSettings,
    pub(in crate::spaces::assistant) request: StartAssistantTaskRequest,
    pub(in crate::spaces::assistant) ids: TaskIds,
    pub(in crate::spaces::assistant) catalog: ToolCatalog,
    pub(in crate::spaces::assistant) session: Session,
    pub(in crate::spaces::assistant) updates: Option<Channel<AssistantTaskUpdate>>,
    pub(in crate::spaces::assistant) last_message: Mutex<Option<String>>,
}

impl Host for Gateway {
    fn dispatch(&self, method: String, params: Value) -> BoxFuture<'_, Result<Value, String>> {
        Box::pin(async move {
            match method.as_str() {
                "assistant.requireRecovery" => {
                    let identity: AssistantExecutionIdentity = decode(params)?;
                    self.require_identity(&identity).await?;
                    ledger::require_reconciled(&self.pool, &self.ids.task_id).await?;
                    Ok(Value::Null)
                }
                "assistant.admitModel" => {
                    let request: AssistantModelAdmission = decode(params)?;
                    self.require_identity(&request.identity).await?;
                    ledger::admit(&self.pool, &self.ids.task_id, request).await
                }
                "assistant.completeModel" => {
                    let request: AssistantModelCompletion = decode(params)?;
                    self.require_process_identity(&request.identity)?;
                    ledger::complete(&self.pool, &self.ids.task_id, request).await
                }
                "assistant.failModel" => {
                    let request: AssistantModelFailure = decode(params)?;
                    self.require_process_identity(&request.identity)?;
                    ledger::fail(&self.pool, &self.ids.task_id, request).await
                }
                "assistant.admitTool" => self.admit_tool(decode(params)?).await,
                "assistant.invokeTool" => {
                    let request: AssistantToolInvocation = decode(params)?;
                    self.require_identity(&request.identity).await?;
                    crate::spaces::assistant::task::tool_execution::invoke(self, request).await
                }
                "assistant.commitProjection" => {
                    let request: AssistantProjection = decode(params)?;
                    self.require_identity(&self.session.identity(&self.ids.run_id))
                        .await?;
                    projection::commit(&self.pool, &self.ids, request).await
                }
                "assistant.contentDelta" => self.delta(decode(params)?).await,
                _ => Err("assistant-host-method-not-found".into()),
            }
        })
    }
}

impl Gateway {
    pub(in crate::spaces::assistant) async fn require_identity(
        &self,
        identity: &AssistantExecutionIdentity,
    ) -> Result<(), String> {
        self.require_process_identity(identity)?;
        let epoch: i64 = sqlx::query_scalar("SELECT control_epoch FROM spaces_assistant_tasks WHERE id=$1 AND run_id=$2 AND segment_id=$3")
            .bind(&self.ids.task_id).bind(&identity.run_id).bind(&identity.thread_id).fetch_one(&self.pool).await.map_err(db::sql_error)?;
        if u64::try_from(epoch).map_err(|_| "assistant-control-epoch-invalid")?
            != identity.control_epoch
        {
            return Err("assistant-execution-superseded".into());
        }
        Ok(())
    }

    fn require_process_identity(
        &self,
        identity: &AssistantExecutionIdentity,
    ) -> Result<(), String> {
        let expected = self.session.identity(&self.ids.run_id);
        if identity.run_id != expected.run_id
            || identity.thread_id != expected.thread_id
            || identity.execution_revision != expected.execution_revision
            || identity.control_epoch != expected.control_epoch
        {
            return Err("assistant-execution-identity-mismatch".into());
        }
        Ok(())
    }

    async fn admit_tool(&self, request: AssistantToolAdmission) -> Result<Value, String> {
        self.require_identity(&request.identity).await?;
        self.require_tool(&request.name, &request.graph_namespace)?;
        if request.sibling_tools.iter().any(|name| name == "task")
            && self
                .catalog
                .by_wire_name
                .get(&request.name)
                .is_some_and(|definition| {
                    definition.impact != crate::spaces::actions::SpaceActionImpact::ReadOnly
                })
        {
            return Err("assistant-research-write-conflict: complete research before publishing business changes".into());
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

    pub(in crate::spaces::assistant) fn require_tool(
        &self,
        name: &str,
        namespace: &str,
    ) -> Result<(), String> {
        let research = namespace.contains('|');
        if let Some(definition) = self.catalog.by_wire_name.get(name) {
            if research && definition.impact != crate::spaces::actions::SpaceActionImpact::ReadOnly
            {
                return Err("assistant-research-write-denied".into());
            }
            return Ok(());
        }
        let allowed = match name {
            "ls" | "read_file" | "glob" | "grep" | "compact_conversation" => true,
            "write_file" | "edit_file" => !research,
            "write_todos" => !research && !matches!(self.request.mode, AssistantMode::Ask),
            "task" => !research,
            _ => false,
        };
        if allowed {
            Ok(())
        } else {
            Err("assistant-tool-not-allowed".into())
        }
    }

    async fn delta(&self, request: AssistantContentDelta) -> Result<Value, String> {
        self.require_identity(&request.identity).await?;
        let mut last = self
            .last_message
            .lock()
            .map_err(|_| "assistant-stream-state-unavailable")?;
        if let Some(channel) = &self.updates {
            if last.as_ref().is_some_and(|id| id != &request.message_id) {
                let _ = channel.send(AssistantTaskUpdate::ContentBoundary);
            }
            let _ = channel.send(AssistantTaskUpdate::ContentDelta {
                task_id: self.ids.task_id.clone(),
                text: request.text,
            });
        }
        *last = Some(request.message_id);
        Ok(Value::Null)
    }
}

pub(in crate::spaces::assistant) fn tool_key(
    identity: &AssistantExecutionIdentity,
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
    format!("assistant:{:x}", Sha256::digest(bytes))
}

fn decode<T: DeserializeOwned>(value: Value) -> Result<T, String> {
    serde_json::from_value(value).map_err(|_| "assistant-invalid-host-parameters".into())
}
