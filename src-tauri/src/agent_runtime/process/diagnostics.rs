use serde::{Deserialize, Serialize};
use tokio::io::AsyncRead;

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
enum AgentDiagnosticEvent {
    AgentRuntimeRequestFailed,
}

#[derive(Deserialize, Serialize)]
enum AgentDiagnosticMethod {
    #[serde(rename = "agentRuntime.control")]
    Control,
    #[serde(rename = "agentRuntime.execute")]
    Execute,
    #[serde(rename = "agentRuntime.initialize")]
    Initialize,
    #[serde(rename = "agentRuntime.projection")]
    Projection,
    #[serde(rename = "agentRuntime.pruneThreads")]
    PruneThreads,
    #[serde(rename = "agentRuntime.shutdown")]
    Shutdown,
    #[serde(rename = "unknown")]
    Unknown,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct AgentDiagnosticFrame {
    file: String,
    line: u32,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct AgentDiagnostic {
    event: AgentDiagnosticEvent,
    method: AgentDiagnosticMethod,
    exception_type: String,
    frames: Vec<AgentDiagnosticFrame>,
}

pub(super) async fn observe(stream: impl AsyncRead + Unpin) {
    if let Err(error) =
        crate::subprocess::diagnostic_lines::consume(stream, 16 * 1024, report).await
    {
        log::warn!("agent runtime diagnostic stream failed: {error}");
    }
}

fn report(line: &[u8]) {
    if let Some(encoded) =
        decode(line).and_then(|diagnostic| serde_json::to_string(&diagnostic).ok())
    {
        log::warn!("agent runtime diagnostic {encoded}");
    }
}

pub(super) fn decode(line: &[u8]) -> Option<AgentDiagnostic> {
    let diagnostic: AgentDiagnostic = serde_json::from_slice(line).ok()?;
    if !valid_symbol(&diagnostic.exception_type)
        || diagnostic.frames.len() > 8
        || diagnostic
            .frames
            .iter()
            .any(|frame| !valid_symbol(&frame.file) || frame.line == 0)
    {
        return None;
    }
    Some(diagnostic)
}

fn valid_symbol(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"_.<>-".contains(&byte))
}
