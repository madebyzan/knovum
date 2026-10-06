use super::SubprocessOutput;
use std::{
    collections::VecDeque,
    io::{BufRead, BufReader, Read},
    process::Child,
    sync::mpsc,
    thread,
    time::Duration,
};
use tokio_util::sync::CancellationToken;

const POLL_INTERVAL: Duration = Duration::from_millis(40);
const DIAGNOSTIC_LIMIT: usize = 80;
const OUTPUT_BUFFER_CAPACITY: usize = 128;
const MAX_OUTPUT_LINE_BYTES: usize = 64 * 1024;

enum StreamLine {
    Stdout(String),
    Stderr(String),
    Error(String),
}

pub(crate) fn wait_with_stderr(
    child: Child,
    cancellation: &CancellationToken,
    cancellation_error: &str,
    mut on_line: impl FnMut(&str) -> Result<(), String>,
) -> Result<SubprocessOutput, String> {
    wait_with_streams(
        child,
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
        cancellation,
        cancellation_error,
        true,
        on_stdout,
        on_stderr,
    )
}

fn wait_with_streams(
    mut child: Child,
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
            return abort_process(child, readers, receiver, cancellation_error.into());
        }
        for _ in 0..OUTPUT_BUFFER_CAPACITY {
            let Ok(line) = receiver.try_recv() else {
                break;
            };
            if let Err(error) = consume_line(line, &mut diagnostics, &mut on_stdout, &mut on_stderr)
            {
                return abort_process(child, readers, receiver, error);
            }
        }
        match child.try_wait() {
            Ok(Some(status)) => {
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
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
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
    })
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
    readers: Vec<thread::JoinHandle<()>>,
    receiver: mpsc::Receiver<StreamLine>,
    error: String,
) -> Result<SubprocessOutput, String> {
    let _ = child.kill();
    let _ = child.wait();
    drop(receiver);
    let _ = join_readers(readers);
    Err(error)
}

fn join_readers(readers: Vec<thread::JoinHandle<()>>) -> Result<(), String> {
    for reader in readers {
        reader
            .join()
            .map_err(|_| "process output reader panicked")?;
    }
    Ok(())
}

#[cfg(test)]
#[path = "tests/runner.rs"]
mod tests;
