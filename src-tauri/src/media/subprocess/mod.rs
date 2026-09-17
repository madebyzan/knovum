use std::{
    collections::VecDeque,
    io::{BufRead, BufReader, Read},
    process::{Child, ExitStatus},
    sync::mpsc,
    thread,
    time::Duration,
};

use tokio_util::sync::CancellationToken;

const POLL_INTERVAL: Duration = Duration::from_millis(40);
const DIAGNOSTIC_LIMIT: usize = 80;

pub(crate) struct SubprocessOutput {
    pub status: ExitStatus,
    pub diagnostics: Vec<String>,
}

enum StreamLine {
    Stdout(String),
    Stderr(String),
    Error(String),
}

pub fn wait_with_stderr(
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

pub fn wait_with_output(
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
    let (sender, receiver) = mpsc::channel();
    let mut readers = vec![spawn_reader(stderr, sender.clone(), StreamLine::Stderr)];
    if let Some(stdout) = stdout {
        readers.push(spawn_reader(stdout, sender.clone(), StreamLine::Stdout));
    }
    drop(sender);
    let mut diagnostics = VecDeque::with_capacity(DIAGNOSTIC_LIMIT);
    loop {
        if cancellation.is_cancelled() {
            return abort_process(child, readers, cancellation_error.into());
        }
        while let Ok(line) = receiver.try_recv() {
            if let Err(error) = consume_line(line, &mut diagnostics, &mut on_stdout, &mut on_stderr)
            {
                return abort_process(child, readers, error);
            }
        }
        let status = match child.try_wait() {
            Ok(status) => status,
            Err(error) => {
                return abort_process(
                    child,
                    readers,
                    format!("failed to query child process: {error}"),
                );
            }
        };
        if let Some(status) = status {
            join_readers(readers)?;
            while let Ok(line) = receiver.try_recv() {
                consume_line(line, &mut diagnostics, &mut on_stdout, &mut on_stderr)?;
            }
            return Ok(SubprocessOutput {
                status,
                diagnostics: diagnostics.into(),
            });
        }
        thread::sleep(POLL_INTERVAL);
    }
}

fn spawn_reader(
    stream: impl Read + Send + 'static,
    sender: mpsc::Sender<StreamLine>,
    map_line: fn(String) -> StreamLine,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        for line in BufReader::new(stream).lines() {
            let value = match line {
                Ok(line) => map_line(line),
                Err(error) => StreamLine::Error(format!("failed to read process output: {error}")),
            };
            if sender.send(value).is_err() {
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
            push_diagnostic(diagnostics, line);
        }
        StreamLine::Error(error) => return Err(error),
    }
    Ok(())
}

fn abort_process(
    mut child: Child,
    readers: Vec<thread::JoinHandle<()>>,
    error: String,
) -> Result<SubprocessOutput, String> {
    let _ = child.kill();
    let _ = child.wait();
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

pub fn diagnostic_error(lines: &[String], default_message: &str) -> String {
    let message = lines.join("\n").trim().to_owned();
    if message.is_empty() {
        default_message.to_owned()
    } else {
        message
    }
}

fn push_diagnostic(lines: &mut VecDeque<String>, line: String) {
    if lines.len() == DIAGNOSTIC_LIMIT {
        lines.pop_front();
    }
    lines.push_back(line);
}

mod progress;
pub(crate) use progress::parse_progress;

#[cfg(test)]
mod tests;
