use super::contracts::{SaveWorkflowRunRequest, SaveWorkflowStep, WorkflowStepStatus};
use super::validation::validate;

fn request(
    subtitle_status: WorkflowStepStatus,
    subtitle_error: &str,
    video_status: WorkflowStepStatus,
    video_error: &str,
) -> SaveWorkflowRunRequest {
    SaveWorkflowRunRequest {
        id: "run".into(),
        space_id: "space".into(),
        workflow_id: "video-with-subtitles".into(),
        workflow_version: 1,
        steps: vec![
            SaveWorkflowStep {
                id: "subtitles".into(),
                action: "extract-subtitles".into(),
                status: subtitle_status,
                output_node_ids: Vec::new(),
                error: subtitle_error.into(),
            },
            SaveWorkflowStep {
                id: "video".into(),
                action: "render-subtitled-video".into(),
                status: video_status,
                output_node_ids: Vec::new(),
                error: video_error.into(),
            },
        ],
    }
}

#[test]
fn derives_terminal_run_state_from_steps() {
    let completed = request(
        WorkflowStepStatus::Succeeded,
        "",
        WorkflowStepStatus::Succeeded,
        "",
    );
    assert_eq!(validate(&completed).expect("completed").status, "completed");

    let failed = request(
        WorkflowStepStatus::Failed,
        "transcription failed",
        WorkflowStepStatus::Pending,
        "",
    );
    let state = validate(&failed).expect("failed");
    assert_eq!(state.status, "failed");
    assert_eq!(state.error, "transcription failed");
}

#[test]
fn rejects_step_error_that_disagrees_with_status() {
    let invalid = request(
        WorkflowStepStatus::Running,
        "unexpected error",
        WorkflowStepStatus::Pending,
        "",
    );
    assert!(validate(&invalid).is_err());
}

#[test]
fn rejects_running_work_after_a_step_fails() {
    let invalid = request(
        WorkflowStepStatus::Failed,
        "transcription failed",
        WorkflowStepStatus::Running,
        "",
    );

    assert!(validate(&invalid).is_err());
}
