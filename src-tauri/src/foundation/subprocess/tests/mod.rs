use std::{
    process::{Command, Stdio},
    sync::mpsc,
    thread,
    time::Duration,
};

use tokio_util::sync::CancellationToken;

use super::{wait_with_output, wait_with_stderr};

#[test]
fn stderr_wait_returns_after_process_exit() {
    let mut command = completed_process_command();
    let child = command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .expect("test process should start");
    let (sender, receiver) = mpsc::channel();
    thread::spawn(move || {
        let output = wait_with_stderr(child, &CancellationToken::new(), "cancelled", |_| Ok(()));
        let _ = sender.send(output);
    });
    let output = receiver
        .recv_timeout(Duration::from_secs(3))
        .expect("process output collection should not wait on a retained channel sender")
        .expect("test process should complete successfully");
    assert!(output.status.success());
}

#[test]
fn stdout_progress_is_delivered_without_entering_stderr_diagnostics() {
    let mut command = completed_process_command();
    let child = command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("test process should start");
    let mut progress = Vec::new();
    let output = wait_with_output(
        child,
        &CancellationToken::new(),
        "cancelled",
        |line| {
            progress.push(line.to_owned());
            Ok(())
        },
        |_| Ok(()),
    )
    .expect("test process should complete successfully");
    assert!(output.status.success());
    assert_eq!(progress, ["knovum-progress:512:1024"]);
    assert_eq!(output.diagnostics, ["diagnostic"]);
}

#[cfg(unix)]
fn completed_process_command() -> Command {
    let mut command = Command::new("sh");
    command.args([
        "-c",
        "printf 'knovum-progress:512:1024\\n'; printf 'diagnostic\\n' >&2",
    ]);
    command
}

#[cfg(windows)]
fn completed_process_command() -> Command {
    let mut command = Command::new("cmd");
    // cmd echoes a space before a redirect operator and treats a digit glued to the text as
    // literal, so emit the diagnostic with neither: `>&2` duplicates stdout onto stderr.
    command.args(["/C", "echo knovum-progress:512:1024& echo diagnostic>&2"]);
    command
}
