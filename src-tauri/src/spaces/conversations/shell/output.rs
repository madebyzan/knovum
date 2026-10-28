//! What a command leaves for the Agent: its output, read while it runs, and how it ended — in the
//! shape deepagents' own shell backends give a model, so a model reads it as it would any shell.

use std::time::Duration;

use serde_json::{json, Value};
use tokio::io::{AsyncRead, AsyncReadExt};

/// The most a command's output may hand the Agent; anything past it is read and dropped, so a
/// chatty command never stalls on a full pipe.
pub(super) const MAX_OUTPUT_BYTES: usize = 100_000;

pub(super) struct Finished {
    pub output: Captured,
    pub exit: Exit,
}

pub(super) enum Exit {
    Code(i32),
    /// Ended by a signal, which on Unix leaves no exit code.
    Signal,
    /// Stopped because it outlived its timeout.
    TimedOut,
}

pub(super) struct Captured {
    stdout: Vec<u8>,
    stderr: Vec<u8>,
    truncated: bool,
}

/// Reads both streams to their end, keeping at most `MAX_OUTPUT_BYTES` of each.
pub(super) async fn capture(
    stdout: impl AsyncRead + Unpin,
    stderr: impl AsyncRead + Unpin,
) -> Captured {
    let ((stdout, stdout_cut), (stderr, stderr_cut)) = tokio::join!(drain(stdout), drain(stderr));
    Captured {
        stdout,
        stderr,
        truncated: stdout_cut || stderr_cut,
    }
}

async fn drain(mut stream: impl AsyncRead + Unpin) -> (Vec<u8>, bool) {
    let mut kept = Vec::new();
    let mut truncated = false;
    let mut buffer = [0_u8; 8192];
    // A read error ends the stream like its end does: what was read is what the command said.
    while let Ok(read) = stream.read(&mut buffer).await {
        if read == 0 {
            break;
        }
        let room = MAX_OUTPUT_BYTES - kept.len();
        kept.extend_from_slice(&buffer[..read.min(room)]);
        truncated |= read > room;
    }
    (kept, truncated)
}

pub(super) fn result(finished: &Finished, timeout: Duration) -> Value {
    let mut text = text(&finished.output);
    let timed_out = matches!(finished.exit, Exit::TimedOut);
    match finished.exit {
        Exit::Code(0) => {}
        Exit::Code(code) => text.push_str(&format!("\n\nExit code: {code}")),
        Exit::Signal => text.push_str("\n\nThe command was terminated by a signal."),
        Exit::TimedOut => text.push_str(&format!(
            "\n\nError: the command was stopped after {} seconds. Pass a longer timeout if it needs more time.",
            timeout.as_secs()
        )),
    }
    json!({"isError": timed_out, "content": [{"type": "text", "text": text}]})
}

fn text(output: &Captured) -> String {
    let mut parts = Vec::new();
    let stdout = String::from_utf8_lossy(&output.stdout);
    if !stdout.is_empty() {
        parts.push(stdout.into_owned());
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    parts.extend(stderr.lines().map(|line| format!("[stderr] {line}")));
    let mut text = parts.join("\n");
    if text.is_empty() {
        text.push_str("<no output>");
    }
    let mut truncated = output.truncated;
    if text.len() > MAX_OUTPUT_BYTES {
        let mut end = MAX_OUTPUT_BYTES;
        while !text.is_char_boundary(end) {
            end -= 1;
        }
        text.truncate(end);
        truncated = true;
    }
    if truncated {
        text.push_str(&format!(
            "\n\n... Output truncated at {MAX_OUTPUT_BYTES} bytes."
        ));
    }
    text
}

#[cfg(test)]
impl Captured {
    pub(super) fn of(stdout: &str, stderr: &str) -> Self {
        Self {
            stdout: stdout.into(),
            stderr: stderr.into(),
            truncated: false,
        }
    }
}
