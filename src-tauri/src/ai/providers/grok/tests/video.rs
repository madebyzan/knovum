use super::*;

fn job(value: serde_json::Value) -> JobStatusResponse {
    serde_json::from_value(value).expect("valid job payload")
}

#[test]
fn done_job_exposes_the_temporary_video_url() {
    let status = parse_status(job(json!({
        "status": "done",
        "video": { "url": "https://vidgen.x.ai/v1/videos/id/video.mp4" },
    })))
    .expect("done status");
    match status {
        VideoGenerationStatus::Succeeded { video_url } => {
            assert_eq!(video_url, "https://vidgen.x.ai/v1/videos/id/video.mp4")
        }
        other => panic!("unexpected status: {other:?}"),
    }
}

#[test]
fn done_job_without_video_is_a_protocol_error() {
    let error = parse_status(job(json!({ "status": "done" }))).unwrap_err();
    assert_eq!(error.code, "invalid_response");
}

#[test]
fn failed_job_carries_the_provider_message() {
    let status = parse_status(job(json!({
        "status": "failed",
        "error": { "code": "internal_error", "message": "generation failed" },
    })))
    .expect("failed status");
    assert!(matches!(
        status,
        VideoGenerationStatus::Failed { error_message } if error_message == "generation failed"
    ));
}

#[test]
fn expired_job_fails_with_a_default_message() {
    let status = parse_status(job(json!({ "status": "expired" }))).expect("expired status");
    assert!(matches!(
        status,
        VideoGenerationStatus::Failed { error_message } if error_message == "Grok video generation failed"
    ));
}

#[test]
fn pending_job_maps_to_the_polling_state() {
    assert!(matches!(
        parse_status(job(json!({ "status": "pending" }))).expect("status"),
        VideoGenerationStatus::Pending { .. }
    ));
}

#[test]
fn unknown_status_is_a_protocol_error() {
    let error = parse_status(job(json!({ "status": "cancelled" }))).unwrap_err();
    assert_eq!(error.code, "invalid_response");
}
