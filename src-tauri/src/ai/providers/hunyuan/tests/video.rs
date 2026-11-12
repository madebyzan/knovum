use super::*;

fn job(value: serde_json::Value) -> JobStatusResponse {
    serde_json::from_value(value).expect("valid job payload")
}

#[test]
fn queued_and_running_jobs_map_to_the_polling_states() {
    assert!(matches!(
        parse_status(job(json!({ "status": "queued" }))).expect("status"),
        VideoGenerationStatus::Pending { .. }
    ));
    assert!(matches!(
        parse_status(job(json!({ "status": "running" }))).expect("status"),
        VideoGenerationStatus::Processing { .. }
    ));
}

#[test]
fn a_succeeded_job_exposes_the_temporary_signed_video_url() {
    let status = parse_status(job(json!({
        "status": "succeeded",
        "videos": [{ "audio_url": "", "url": "https://aigc-output-video.cos.ap-guangzhou.myqcloud.com/x.mp4?q-sign-algorithm=sha1" }],
    })))
    .expect("succeeded status");
    match status {
        VideoGenerationStatus::Succeeded { video_url } => {
            assert!(video_url.starts_with("https://aigc-output-video."));
        }
        other => panic!("unexpected status: {other:?}"),
    }
}

#[test]
fn succeeded_without_a_video_is_a_protocol_error() {
    let error = parse_status(job(json!({ "status": "succeeded" }))).unwrap_err();
    assert_eq!(error.code, "invalid_response");
}

#[test]
fn a_failed_job_carries_the_provider_message() {
    let status = parse_status(job(json!({
        "status": "failed",
        "error": { "code": "InternalServiceError", "message": "generate video failed" },
    })))
    .expect("failed status");
    assert!(matches!(
        status,
        VideoGenerationStatus::Failed { error_message } if error_message == "generate video failed"
    ));
}

#[test]
fn failed_without_a_message_falls_back_to_a_default() {
    let status = parse_status(job(json!({ "status": "failed" }))).expect("failed status");
    assert!(matches!(
        status,
        VideoGenerationStatus::Failed { error_message } if error_message == "TokenHub video generation failed"
    ));
}

#[test]
fn unknown_status_is_a_protocol_error() {
    let error = parse_status(job(json!({ "status": "cancelled" }))).unwrap_err();
    assert_eq!(error.code, "invalid_response");
}

#[test]
fn a_rejected_submission_reports_the_error_envelope() {
    let submission: SubmissionResponse = serde_json::from_value(json!({
        "task_id": "",
        "status": "failed",
        "error": { "code": "1300", "message": "Trigger the platform strategy" },
    }))
    .expect("rejection envelope");
    assert!(submission.task_id.is_empty());
    let error = job_error(submission.error.expect("error"));
    assert_eq!(error.code, "provider_request_failed");
    assert_eq!(error.message, "Trigger the platform strategy");
}
