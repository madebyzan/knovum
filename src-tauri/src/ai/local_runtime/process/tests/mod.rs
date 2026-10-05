use std::{
    path::PathBuf,
    process::Stdio,
    time::{Instant, SystemTime, UNIX_EPOCH},
};

use tokio::process::Command;
use tokio_util::sync::CancellationToken;

use super::{KillRegistration, ProcessTree, Worker};
use crate::ai::{
    contracts::runtime::{RuntimeCapability, RuntimeDeviceProfile},
    local_runtime::protocol::Protocol,
};

async fn worker() -> Worker {
    let session_id = uuid::Uuid::new_v4().to_string();
    let directory = std::env::temp_dir().join(format!("knovum-worker-send-{session_id}"));
    std::fs::create_dir(&directory).unwrap();
    let mut command = Command::new(std::env::current_exe().unwrap());
    command
        .args([
            "--exact",
            "ai::local_runtime::process::tests::stalled_worker",
            "--ignored",
            "--nocapture",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .kill_on_drop(true);
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        command.as_std_mut().process_group(0);
    }
    #[cfg(windows)]
    let job = super::windows_job::WindowsJob::new().unwrap();
    let mut child = command.spawn().unwrap();
    #[cfg(windows)]
    job.assign(&child).unwrap();
    let process_tree = ProcessTree::new(
        &child,
        #[cfg(windows)]
        job,
    )
    .unwrap();
    let protocol = Protocol::new(child.stdin.take().unwrap(), child.stdout.take().unwrap());
    Worker {
        _kill_registration: KillRegistration::new(&session_id, process_tree.handle()),
        child,
        process_tree,
        memory_reservation: None,
        protocol,
        session_id,
        runtime_digest: "test-runtime".into(),
        model_path: PathBuf::new(),
        capability: RuntimeCapability::Embedding,
        capability_spec_id: "test-model".into(),
        device_profile: RuntimeDeviceProfile {
            device: "cpu".into(),
            execution_profile_id: "test".into(),
            cpu_threads: 1,
        },
        work_directory: directory,
        last_used: Instant::now(),
    }
}

fn deadline_after(ms: u64) -> u64 {
    u64::try_from(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis(),
    )
    .unwrap()
        + ms
}

#[test]
#[ignore = "subprocess fixture; launched by worker send regression tests"]
fn stalled_worker() {
    std::thread::sleep(std::time::Duration::from_secs(30));
}

#[tokio::test]
async fn failed_sends_terminate_the_worker_and_remove_its_workspace() {
    for mode in ["elapsed", "cancelled", "timeout"] {
        let mut worker = worker().await;
        let directory = worker.work_directory.clone();
        let cancellation = CancellationToken::new();
        if mode == "cancelled" {
            cancellation.cancel();
        }
        let deadline = if mode == "elapsed" {
            1
        } else {
            deadline_after(200)
        };
        let params = serde_json::json!({"text": "x".repeat(900_000)});
        let failure = worker
            .send_request(
                "request",
                "embedding.encode",
                &params,
                deadline,
                &cancellation,
                &CancellationToken::new(),
            )
            .await
            .unwrap_err();
        assert_eq!(
            failure,
            if mode == "cancelled" {
                "cancelled"
            } else {
                "deadline_exceeded"
            }
        );
        assert!(worker.child.try_wait().unwrap().is_some());
        assert!(!directory.exists());
        assert!(worker.process_tree.0.is_none());
    }
}

#[tokio::test]
async fn successful_send_preserves_the_worker_and_returns_the_response_deadline() {
    let mut worker = worker().await;
    let deadline = worker
        .send_request(
            "request",
            "runtime.health",
            &serde_json::json!({}),
            deadline_after(5_000),
            &CancellationToken::new(),
            &CancellationToken::new(),
        )
        .await
        .unwrap();
    assert!(deadline > tokio::time::Instant::now());
    assert!(worker.child.try_wait().unwrap().is_none());
    worker.terminate().await;
}
