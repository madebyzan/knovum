use super::{process_tree::ProcessTree, SubprocessOutput};
use std::{
    collections::VecDeque,
    io::{BufRead, BufReader, Read},
    process::{Child, Command},
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};
use tokio_util::sync::CancellationToken;

const POLL_INTERVAL: Duration = Duration::from_millis(40);
const ABORT_READER_JOIN_TIMEOUT: Duration = Duration::from_secs(2);
const DIAGNOSTIC_LIMIT: usize = 80;
const OUTPUT_BUFFER_CAPACITY: usize = 128;
const MAX_OUTPUT_LINE_BYTES: usize = 64 * 1024;

pub(crate) struct ManagedChild {
    child: Child,
    tree: ProcessTree,
}

/// Spawns `command` inside its own process group (Unix) or Job Object with `CREATE_NO_WINDOW`
/// (Windows) so cancellation terminates both the child and any grandchildren it spawned.
pub(crate) fn spawn_managed(command: &mut Command) -> Result<ManagedChild, String> {
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        command.process_group(0);
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
    }
    #[cfg(windows)]
    let job = super::windows_job::WindowsJob::new()?;
    let child = command.spawn().map_err(|error| error.to_string())?;
    let tree = ProcessTree::new_std(
        &child,
        #[cfg(windows)]
        job,
    )?;
    Ok(ManagedChild { child, tree })
}

enum StreamLine {
    Stdout(String),
    Stderr(String),
    Error(String),
}

struct ReaderHandle {
    handle: thread::JoinHandle<()>,
    done: mpsc::Receiver<()>,
}

pub(crate) fn wait_managed_with_stderr(
    managed: ManagedChild,
    cancellation: &CancellationToken,
    cancellation_error: &str,
    mut on_line: impl FnMut(&str) -> Result<(), String>,
) -> Result<SubprocessOutput, String> {
    wait_with_streams(
        managed.child,
        Some(managed.tree),
        cancellation,
        cancellation_error,
        false,
        |_| Ok(()),
        |line| on_line(line),
    )
}

pub(crate) fn wait_managed_with_output(
    managed: ManagedChild,
    cancellation: &CancellationToken,
    cancellation_error: &str,
    on_stdout: impl FnMut(&str) -> Result<(), String>,
    on_stderr: impl FnMut(&str) -> Result<(), String>,
) -> Result<SubprocessOutput, String> {
    wait_with_streams(
        managed.child,
        Some(managed.tree),
        cancellation,
        cancellation_error,
        true,
        on_stdout,
        on_stderr,
    )
}

pub(crate) fn wait_with_stderr(
    child: Child,
    cancellation: &CancellationToken,
    cancellation_error: &str,
    mut on_line: impl FnMut(&str) -> Result<(), String>,
) -> Result<SubprocessOutput, String> {
    wait_with_streams(
        child,
        None,
        cancellation,
        cancellation_error,
        false,
        |_| Ok(()),
        |line| on_line(line),
    )
}

pub(crate) fn wait_with_output(
    child: Child,
    cancellation: &CancellationToken,
    cancellation_error: &str,
    on_stdout: impl FnMut(&str) -> Result<(), String>,
    on_stderr: impl FnMut(&str) -> Result<(), String>,
) -> Result<SubprocessOutput, String> {
    wait_with_streams(
        child,
        None,
        cancellation,
        cancellation_error,
        true,
        on_stdout,
        on_stderr,
    )
}

fn wait_with_streams(
    mut child: Child,
    mut tree: Option<ProcessTree>,
    cancellation: &CancellationToken,
    cancellation_error: &str,
    read_stdout: bool,
    mut on_stdout: impl FnMut(&str) -> Result<(), String>,
    mut on_stderr: impl FnMut(&str) -> Result<(), String>,
) -> Result<SubprocessOutput, String> {
    let stderr = child
        .stderr
        .take()
        .ok_or("child process stderr is unavailable")?;
    let stdout = if read_stdout {
        Some(
            child
                .stdout
                .take()
                .ok_or("child process stdout is unavailable")?,
        )
    } else {
        None
    };
    let (sender, receiver) = mpsc::sync_channel(OUTPUT_BUFFER_CAPACITY);
    let mut readers = vec![spawn_reader(stderr, sender.clone(), StreamLine::Stderr)];
    if let Some(stdout) = stdout {
        readers.push(spawn_reader(stdout, sender.clone(), StreamLine::Stdout));
    }
    drop(sender);
    let mut diagnostics = VecDeque::with_capacity(DIAGNOSTIC_LIMIT);
    loop {
        if cancellation.is_cancelled() {
            return abort_process(
                child,
                tree.as_mut(),
                readers,
                receiver,
                cancellation_error.into(),
            );
        }
        for _ in 0..OUTPUT_BUFFER_CAPACITY {
            let Ok(line) = receiver.try_recv() else {
                break;
            };
            if let Err(error) = consume_line(line, &mut diagnostics, &mut on_stdout, &mut on_stderr)
            {
                return abort_process(child, tree.as_mut(), readers, receiver, error);
            }
        }
        match child.try_wait() {
            Ok(Some(status)) => {
                if let Some(tree) = tree.as_mut() {
                    tree.terminate();
                }
                while let Ok(line) = receiver.recv() {
                    consume_line(line, &mut diagnostics, &mut on_stdout, &mut on_stderr)?;
                }
                join_readers(readers)?;
                return Ok(SubprocessOutput {
                    status,
                    diagnostics: diagnostics.into(),
                });
            }
            Ok(None) => thread::sleep(POLL_INTERVAL),
            Err(error) => {
                return abort_process(
                    child,
                    tree.as_mut(),
                    readers,
                    receiver,
                    format!("failed to query child process: {error}"),
                )
            }
        }
    }
}

fn spawn_reader(
    stream: impl Read + Send + 'static,
    sender: mpsc::SyncSender<StreamLine>,
    map_line: fn(String) -> StreamLine,
) -> ReaderHandle {
    let (done_tx, done) = mpsc::channel();
    let handle = thread::spawn(move || {
        let mut reader = BufReader::new(stream);
        loop {
            let mut bytes = Vec::new();
            let read = reader
                .by_ref()
                .take((MAX_OUTPUT_LINE_BYTES + 1) as u64)
                .read_until(b'\n', &mut bytes);
            let value = match read {
                Ok(0) => break,
                Ok(_) if bytes.len() > MAX_OUTPUT_LINE_BYTES => {
                    StreamLine::Error("process output line exceeded the size limit".into())
                }
                Ok(_) => {
                    if bytes.last() == Some(&b'\n') {
                        bytes.pop();
                        if bytes.last() == Some(&b'\r') {
                            bytes.pop();
                        }
                    }
                    match String::from_utf8(bytes) {
                        Ok(line) => map_line(line),
                        Err(error) => StreamLine::Error(format!("invalid process output: {error}")),
                    }
                }
                Err(error) => StreamLine::Error(format!("failed to read process output: {error}")),
            };
            let failed = matches!(value, StreamLine::Error(_));
            if sender.send(value).is_err() || failed {
                break;
            }
        }
        let _ = done_tx.send(());
    });
    ReaderHandle { handle, done }
}

fn consume_line(
    line: StreamLine,
    diagnostics: &mut VecDeque<String>,
    on_stdout: &mut impl FnMut(&str) -> Result<(), String>,
    on_stderr: &mut impl FnMut(&str) -> Result<(), String>,
) -> Result<(), String> {
    match line {
        StreamLine::Stdout(line) => on_stdout(&line)?,
        StreamLine::Stderr(line) => {
            on_stderr(&line)?;
            if diagnostics.len() == DIAGNOSTIC_LIMIT {
                diagnostics.pop_front();
            }
            diagnostics.push_back(line);
        }
        StreamLine::Error(error) => return Err(error),
    }
    Ok(())
}

fn abort_process(
    mut child: Child,
    tree: Option<&mut ProcessTree>,
    readers: Vec<ReaderHandle>,
    receiver: mpsc::Receiver<StreamLine>,
    error: String,
) -> Result<SubprocessOutput, String> {
    if let Some(tree) = tree {
        tree.terminate();
    }
    let _ = child.kill();
    let _ = child.wait();
    drop(receiver);
    join_readers_bounded(readers, ABORT_READER_JOIN_TIMEOUT);
    Err(error)
}

fn join_readers(readers: Vec<ReaderHandle>) -> Result<(), String> {
    for reader in readers {
        reader
            .handle
            .join()
            .map_err(|_| "process output reader panicked")?;
    }
    Ok(())
}

fn join_readers_bounded(readers: Vec<ReaderHandle>, timeout: Duration) {
    let deadline = Instant::now() + timeout;
    for reader in readers {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if reader.done.recv_timeout(remaining).is_ok() {
            let _ = reader.handle.join();
        }
    }
}

#[cfg(test)]
#[path = "tests/runner.rs"]
mod tests;
