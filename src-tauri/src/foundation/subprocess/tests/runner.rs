use std::sync::mpsc;

use super::{spawn_reader, StreamLine, MAX_OUTPUT_LINE_BYTES};

#[test]
fn reader_rejects_a_line_above_the_output_budget() {
    let (sender, receiver) = mpsc::sync_channel(2);
    let reader = spawn_reader(
        std::io::Cursor::new(vec![b'x'; MAX_OUTPUT_LINE_BYTES + 1]),
        sender,
        StreamLine::Stderr,
    );
    assert!(matches!(receiver.recv(), Ok(StreamLine::Error(_))));
    reader.handle.join().expect("reader thread must finish");
}

#[cfg(unix)]
#[test]
fn cancelling_managed_subprocess_kills_grandchildren_without_deadlocking_readers() {
    use std::{
        process::{Command, Stdio},
        time::{Duration, Instant},
    };
    use tokio_util::sync::CancellationToken;

    let mut command = Command::new("/bin/sh");
    command
        .args(["-c", "sleep 30 & echo ready; sleep 30"])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let managed = super::spawn_managed(&mut command).expect("spawn managed");
    let cancellation = CancellationToken::new();
    let cancel_trigger = cancellation.clone();
    let started = Instant::now();
    let outcome = super::wait_managed_with_output(
        managed,
        &cancellation,
        "cancelled",
        move |line| {
            if line == "ready" {
                cancel_trigger.cancel();
            }
            Ok(())
        },
        |_| Ok(()),
    );
    assert_eq!(outcome.err().as_deref(), Some("cancelled"));
    assert!(
        started.elapsed() < Duration::from_secs(5),
        "cancellation deadlocked waiting on grandchild pipe"
    );
}
