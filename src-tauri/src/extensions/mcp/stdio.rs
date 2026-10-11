use crate::foundation::subprocess::process_tree::ProcessTree;
use std::{collections::BTreeMap, process::Stdio};
use tokio::{
    process::{Child, Command},
    process::{ChildStdin, ChildStdout},
};

pub(super) struct StdioSession {
    pub child: Child,
    pub tree: ProcessTree,
}

pub(super) fn start(
    executable: &str,
    args: &[String],
    cwd: Option<&str>,
    environment: &BTreeMap<String, String>,
    secrets: &BTreeMap<String, String>,
) -> Result<(StdioSession, ChildStdout, ChildStdin), String> {
    let mut command = Command::new(executable);
    command
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .kill_on_drop(true)
        .env_clear();
    #[cfg(windows)]
    for name in ["SYSTEMROOT", "WINDIR", "TEMP", "TMP"] {
        if let Some(value) = std::env::var_os(name) {
            command.env(name, value);
        }
    }
    #[cfg(unix)]
    command.process_group(0);
    if let Some(cwd) = cwd {
        command.current_dir(cwd);
    }
    command.envs(environment).envs(secrets);
    #[cfg(windows)]
    let job = crate::foundation::subprocess::windows_job::WindowsJob::new()?;
    let mut child = command.spawn().map_err(|_| "mcp-process-start-failed")?;
    #[cfg(windows)]
    job.assign(&child)?;
    let tree = ProcessTree::new(
        &child,
        #[cfg(windows)]
        job,
    )?;
    let stdout = child
        .stdout
        .take()
        .ok_or("mcp-process-stdout-unavailable")?;
    let stdin = child.stdin.take().ok_or("mcp-process-stdin-unavailable")?;
    Ok((StdioSession { child, tree }, stdout, stdin))
}

pub(super) fn transport(
    stdout: tokio::process::ChildStdout,
    stdin: tokio::process::ChildStdin,
) -> impl rmcp::transport::Transport<rmcp::RoleClient> {
    use futures::StreamExt;
    use rmcp::{
        model::{ClientJsonRpcMessage, ServerJsonRpcMessage},
        transport::{async_rw::JsonRpcMessageCodec, sink_stream::SinkStreamTransport},
    };
    use tokio_util::codec::{FramedRead, FramedWrite};
    let input = FramedRead::new(
        stdout,
        JsonRpcMessageCodec::<ServerJsonRpcMessage>::new_with_max_length(8 * 1024 * 1024),
    )
    .take_while(|message| futures::future::ready(message.is_ok()))
    .map(|message| message.expect("accepted MCP frame"));
    let output = FramedWrite::new(
        stdin,
        JsonRpcMessageCodec::<ClientJsonRpcMessage>::new_with_max_length(8 * 1024 * 1024),
    );
    SinkStreamTransport::new(output, input)
}
