use std::{
    path::PathBuf,
    time::{Duration, Instant},
};

use serde_json::{json, Value};

use super::{
    output::{self, Captured, Exit, Finished, MAX_OUTPUT_BYTES},
    prepare,
};

/// A directory under the build's target directory, not the system temporary directory, because a
/// confined command may write temporary directories and a test must see where it may not.
struct Scratch(PathBuf);

impl Scratch {
    fn new() -> Self {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("target/shell-tests")
            .join(uuid::Uuid::new_v4().to_string());
        std::fs::create_dir_all(&root).expect("scratch directory");
        Self(root)
    }

    fn workspace(&self) -> PathBuf {
        self.0.join("workspace")
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn text(result: &Value) -> &str {
    result["content"][0]["text"].as_str().expect("text result")
}

async fn run(scratch: &Scratch, command: &str, timeout: u64) -> Value {
    prepare(
        scratch.workspace(),
        json!({"command": command, "timeout": timeout}),
    )
    .expect("valid command")
    .execute(Duration::from_secs(600))
    .await
    .expect("command ran")
}

#[cfg(unix)]
const WRITE_MARKER: &str = "echo knovum > made.txt";
#[cfg(windows)]
const WRITE_MARKER: &str = "Set-Content -Path made.txt -Value knovum -Encoding ascii";

#[test]
fn a_command_needs_a_command_line_and_a_bounded_timeout() {
    for arguments in [
        json!({}),
        json!({"command": ""}),
        json!({"command": "ls", "timeout": 0}),
        json!({"command": "ls", "timeout": 601}),
        json!({"command": "ls", "cwd": "/"}),
    ] {
        assert!(
            prepare(PathBuf::from("workspace"), arguments.clone()).is_err(),
            "{arguments} was accepted"
        );
    }
    assert!(prepare(PathBuf::from("workspace"), json!({"command": "ls"})).is_ok());
}

#[test]
fn only_a_confined_command_runs_without_the_users_approval() {
    let expected = if cfg!(target_os = "macos") {
        crate::extensions::contracts::McpToolPolicy::Allow
    } else {
        crate::extensions::contracts::McpToolPolicy::Confirm
    };
    assert_eq!(super::policy(), expected);
    let instructions = super::instructions();
    assert!(instructions.contains(super::TOOL_NAME));
    assert!(instructions.contains(super::platform::SYSTEM));
}

#[test]
fn output_keeps_errors_apart_and_says_how_the_command_ended() {
    let finished = Finished {
        output: Captured::of("done\n", "warning\n"),
        exit: Exit::Code(2),
    };
    let result = output::result(&finished, Duration::from_secs(5));
    assert_eq!(text(&result), "done\n\n[stderr] warning\n\nExit code: 2");
    assert_eq!(result["isError"], false, "a failing command still ran");
    let silent = Finished {
        output: Captured::of("", ""),
        exit: Exit::Code(0),
    };
    assert_eq!(
        text(&output::result(&silent, Duration::from_secs(5))),
        "<no output>"
    );
}

#[test]
fn output_past_the_limit_is_cut_on_a_character_and_says_so() {
    let finished = Finished {
        output: Captured::of(&"é".repeat(MAX_OUTPUT_BYTES), ""),
        exit: Exit::Code(0),
    };
    let result = output::result(&finished, Duration::from_secs(5));
    let (kept, notice) = text(&result)
        .split_once("\n\n... Output truncated")
        .expect("truncation notice");
    assert!(kept.len() <= MAX_OUTPUT_BYTES);
    assert!(kept.chars().all(|character| character == 'é'));
    assert!(notice.contains(&MAX_OUTPUT_BYTES.to_string()));
}

#[tokio::test]
async fn a_command_runs_in_its_working_directory_and_reports_how_it_ended() {
    let scratch = Scratch::new();
    let written = run(&scratch, WRITE_MARKER, 60).await;
    assert_eq!(written["isError"], false, "{written}");
    let marker = std::fs::read_to_string(scratch.workspace().join("made.txt")).expect("marker");
    assert_eq!(marker.trim(), "knovum");
    let failed = run(&scratch, "exit 3", 60).await;
    assert!(text(&failed).ends_with("Exit code: 3"), "{failed}");
}

#[tokio::test]
async fn a_command_that_outlives_its_timeout_is_stopped() {
    let scratch = Scratch::new();
    let started = Instant::now();
    let stopped = run(&scratch, "sleep 30", 1).await;
    assert!(started.elapsed() < Duration::from_secs(15));
    assert_eq!(stopped["isError"], true);
    assert!(
        text(&stopped).contains("stopped after 1 seconds"),
        "{stopped}"
    );
}

#[cfg(unix)]
#[tokio::test]
async fn every_process_a_command_started_ends_with_it() {
    let scratch = Scratch::new();
    let started = Instant::now();
    let finished = run(&scratch, "sleep 30 & echo started", 60).await;
    assert!(
        started.elapsed() < Duration::from_secs(15),
        "a background process kept the command open"
    );
    assert_eq!(text(&finished).trim(), "started");
}

/// Cargo hands a test its `CARGO_*` variables at run time, as Knovum's process carries keys and
/// tokens of its own: none may reach a command.
#[cfg(unix)]
#[tokio::test]
async fn a_command_sees_none_of_knovums_own_environment() {
    let (name, _) = std::env::vars()
        .find(|(name, value)| name.starts_with("CARGO_") && !value.is_empty())
        .expect("cargo sets CARGO_* variables for a test");
    let scratch = Scratch::new();
    let probed = run(&scratch, &format!("echo \"[${{{name}}}]\""), 60).await;
    assert_eq!(text(&probed).trim(), "[]", "{name} reached the command");
}

#[cfg(target_os = "macos")]
#[tokio::test]
async fn a_confined_command_writes_only_its_working_directory() {
    let scratch = Scratch::new();
    let outside = scratch.0.join("outside.txt");
    run(
        &scratch,
        &format!("echo in > inside.txt; echo out > '{}'", outside.display()),
        60,
    )
    .await;
    assert!(scratch.workspace().join("inside.txt").exists());
    assert!(
        !outside.exists(),
        "Seatbelt let a command write outside its working directory"
    );
}
