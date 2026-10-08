#![cfg(unix)]

use std::path::Path;
use tokio::process::Command;
use tokio_util::sync::CancellationToken;

fn command(script: &str, path: &Path) -> Command {
    let mut command = Command::new("/bin/sh");
    command.args(["-c", script, "document-fixture"]).arg(path);
    command
}

async fn wait_pid(path: &Path) -> i32 {
    for _ in 0..100 {
        if let Ok(pid) = tokio::fs::read_to_string(path).await {
            if let Ok(pid) = pid.trim().parse() {
                return pid;
            }
        }
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
    panic!("document worker did not start");
}

fn is_alive(pid: i32) -> bool {
    unsafe extern "C" {
        fn kill(pid: i32, signal: i32) -> i32;
    }
    unsafe { kill(pid, 0) == 0 }
}

#[tokio::test]
async fn cancellation_reaps_blocked_worker_before_return_and_capacity_release() {
    let path = std::env::temp_dir().join(format!("knovum-document-pid-{}", uuid::Uuid::new_v4()));
    let cancellation = CancellationToken::new();
    let token = cancellation.clone();
    let capacity = std::sync::Arc::new(tokio::sync::Semaphore::new(1));
    let permit = capacity.clone().acquire_owned().await.unwrap();
    let child = command("printf '%s' $$ > \"$1\"; sleep 120", &path);
    let task = tokio::spawn(async move {
        let _permit = permit;
        super::process::run(child, vec![], &token).await
    });
    let pid = wait_pid(&path).await;
    assert!(is_alive(pid));
    assert_eq!(capacity.available_permits(), 0);
    cancellation.cancel();
    let result = tokio::time::timeout(std::time::Duration::from_secs(5), task)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(result.unwrap_err().code, "cancelled");
    assert!(
        !is_alive(pid),
        "worker must exit before returning cancellation"
    );
    assert_eq!(capacity.available_permits(), 1);
    std::fs::remove_file(path).unwrap();
}

#[tokio::test]
async fn malformed_response_terminates_worker_and_never_falls_back_to_rendering() {
    let path = std::env::temp_dir().join(format!("knovum-document-pid-{}", uuid::Uuid::new_v4()));
    let command = command(
        "printf '%s' $$ > \"$1\"; printf '\\377\\377\\377\\377'; sleep 120",
        &path,
    );
    let result = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        super::process::run(command, vec![], &CancellationToken::new()),
    )
    .await
    .unwrap();
    assert_eq!(result.unwrap_err().code, "document_worker_failed");
    assert!(!is_alive(wait_pid(&path).await));
    std::fs::remove_file(path).unwrap();
}

#[tokio::test]
async fn worker_crash_is_an_explicit_failure() {
    let result = super::process::run(
        command("exit 7", Path::new("unused")),
        vec![],
        &CancellationToken::new(),
    )
    .await;
    assert_eq!(result.unwrap_err().code, "document_worker_failed");
}
