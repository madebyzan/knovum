//! What a recorded tool operation reaches once it may run. Every operation an Agent asks for takes
//! the same course — recorded, approved when its policy asks, marked dispatching, settled with a
//! receipt — whatever finally carries it out.

use std::{path::PathBuf, time::Duration};

use serde_json::Value;
use sqlx::SqlitePool;
use tauri::AppHandle;

use crate::{
    extensions::{
        contracts::{AgentExtensionSnapshot, AgentExtensionTool, AgentToolBinding, McpToolPolicy},
        mcp::invocation::{InvocationObservation, PreparedInvocation},
    },
    spaces::conversations::shell,
};

/// How long an MCP server may take to answer once a call is dispatched.
const MCP_RESPONSE_TIMEOUT: Duration = Duration::from_secs(120);

pub(crate) enum ToolTarget<'a> {
    /// A tool of a connected MCP server, frozen in the Run's extension snapshot.
    Mcp {
        snapshot: &'a AgentExtensionSnapshot,
        tool: &'a AgentExtensionTool,
    },
    /// A command line run in the built-in Agent's working directory.
    Command { workspace: PathBuf },
}

pub(super) enum Prepared {
    Mcp(PreparedInvocation),
    Command(shell::PreparedCommand),
}

impl ToolTarget<'_> {
    pub(super) fn name(&self) -> &str {
        match self {
            Self::Mcp { tool, .. } => &tool.name,
            Self::Command { .. } => shell::TOOL_NAME,
        }
    }

    pub(super) fn binding(&self) -> AgentToolBinding {
        match self {
            Self::Mcp { tool, .. } => tool.binding.clone(),
            Self::Command { .. } => AgentToolBinding::Command,
        }
    }

    pub(super) fn read_only(&self) -> bool {
        match self {
            Self::Mcp { tool, .. } => tool.read_only,
            Self::Command { .. } => false,
        }
    }

    pub(super) fn policy(&self) -> McpToolPolicy {
        match self {
            Self::Mcp { tool, .. } => tool.policy,
            Self::Command { .. } => shell::policy(),
        }
    }

    /// An MCP tool is only as current as the extensions it came from.
    pub(super) async fn require_current(&self, app: &AppHandle) -> Result<(), String> {
        match self {
            Self::Mcp { snapshot, .. } => {
                crate::extensions::snapshots::require_current(app, snapshot).await
            }
            Self::Command { .. } => Ok(()),
        }
    }

    pub(super) async fn prepare(
        &self,
        app: &AppHandle,
        pool: &SqlitePool,
        run_id: &str,
        arguments: Value,
    ) -> Result<Prepared, String> {
        match self {
            Self::Mcp { snapshot, tool } => {
                let interaction = std::sync::Arc::new(super::interactions::RunMcpInteraction {
                    pool: pool.clone(),
                    run_id: run_id.into(),
                });
                crate::extensions::mcp::invocation::prepare(
                    app,
                    snapshot,
                    tool,
                    arguments,
                    Some(interaction),
                )
                .await
                .map(Prepared::Mcp)
            }
            Self::Command { workspace } => {
                shell::prepare(workspace.clone(), arguments).map(Prepared::Command)
            }
        }
    }
}

impl Prepared {
    /// Carries the operation out within `remaining`, the time its Run has left.
    pub(super) async fn execute(
        self,
        operation_id: &str,
        remaining: Duration,
    ) -> InvocationObservation {
        match self {
            Self::Mcp(prepared) => {
                prepared
                    .execute(operation_id, remaining.min(MCP_RESPONSE_TIMEOUT))
                    .await
            }
            // A command always ends observed: Knovum ran it and saw it finish or stopped it, or
            // it never started.
            Self::Command(command) => InvocationObservation::Received(
                command
                    .execute(remaining)
                    .await
                    .unwrap_or_else(super::receipts::failure),
            ),
        }
    }
}
