//! The built-in Agent's shell: one command line at a time, run in the conversation's working
//! directory. Knovum runs the command, not the Agent's process, so a command takes the course
//! every external effect takes (`tool_operations`): it is recorded before it runs, approved when it
//! is not confined, settled with a receipt that a replay returns instead of running it again, and
//! killed with everything it started when its Run stops.
//!
//! Where the platform can confine a command to the working directory (macOS Seatbelt), commands
//! run as the Agent asks for them; elsewhere a command runs with the user's own permissions, so
//! the user approves each one first. Either way it sees only an allowlisted environment — never
//! Knovum's credentials or the user's API keys.

use std::{path::PathBuf, time::Duration};

use serde::Deserialize;
use serde_json::{json, Value};

use crate::{ai::contracts::ModelToolDefinition, extensions::contracts::McpToolPolicy};

mod output;
mod platform;

pub(crate) const TOOL_NAME: &str = "run_command";
const DEFAULT_TIMEOUT_SECONDS: u64 = 120;
const MAX_TIMEOUT_SECONDS: u64 = 600;

pub(super) fn definition() -> ModelToolDefinition {
    ModelToolDefinition {
        name: TOOL_NAME.into(),
        description: "Run one command line in your working directory with the shell your instructions name, and return its output and exit code. Use it for computation and for processing files the file tools cannot handle; read, write and edit files with the file tools. The command gets no input, and every process it starts ends with it.".into(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "command": {"type": "string", "minLength": 1, "description": "The command line to run."},
                "timeout": {
                    "type": "integer", "minimum": 1, "maximum": MAX_TIMEOUT_SECONDS,
                    "description": format!("Seconds to wait before the command is stopped; {DEFAULT_TIMEOUT_SECONDS} when omitted.")
                }
            },
            "required": ["command"],
            "additionalProperties": false
        }),
    }
}

/// A confined command can only change the working directory, so it runs as asked; any other runs
/// with the user's permissions, which only the user can lend.
pub(crate) fn policy() -> McpToolPolicy {
    if platform::CONFINED {
        McpToolPolicy::Allow
    } else {
        McpToolPolicy::Confirm
    }
}

/// What the built-in Agent is told about its working directory and the shell.
pub(super) fn instructions() -> String {
    let confinement = if platform::CONFINED {
        "Commands run in a sandbox: they may read the computer's files but write only inside the working directory and temporary directories, and they have no network access, so downloads and package installs fail."
    } else {
        "Commands run with the user's own permissions, so the user approves each one before it runs: run only what the task needs, and prefer the file tools for reading and writing files."
    };
    format!(
        "## Working directory\n\nYour file tools and {TOOL_NAME} share one working directory that belongs to this conversation and keeps its files across turns. The file tools address it as `/`, for example `/data/report.csv`; commands run inside it, so give them relative paths such as `data/report.csv`. Only you can see it: the user cannot open its files, so never link to them — put results in your answer or publish them through Knovum Actions. The computer runs {}, and {TOOL_NAME} uses {}. {confinement}",
        platform::SYSTEM,
        platform::SHELL,
    )
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CommandArguments {
    command: String,
    timeout: Option<u64>,
}

pub(crate) struct PreparedCommand {
    workspace: PathBuf,
    command: String,
    timeout: Duration,
}

pub(crate) fn prepare(workspace: PathBuf, arguments: Value) -> Result<PreparedCommand, String> {
    jsonschema::validator_for(&definition().input_schema)
        .map_err(|error| error.to_string())?
        .validate(&arguments)
        .map_err(|error| format!("command-arguments-invalid: {error}"))?;
    let arguments: CommandArguments = serde_json::from_value(arguments)
        .map_err(|error| format!("command-arguments-invalid: {error}"))?;
    Ok(PreparedCommand {
        workspace,
        command: arguments.command,
        timeout: Duration::from_secs(arguments.timeout.unwrap_or(DEFAULT_TIMEOUT_SECONDS)),
    })
}

impl PreparedCommand {
    /// Runs the command for at most its own timeout and `remaining`, the time its Run has left,
    /// and returns the tool response the Agent reads, or why the command could not run.
    pub(crate) async fn execute(self, remaining: Duration) -> Result<Value, String> {
        let timeout = self.timeout.min(remaining);
        let finished = platform::run(&self.workspace, &self.command, timeout).await?;
        Ok(output::tool_response(&finished, timeout))
    }
}

#[cfg(test)]
mod tests;
