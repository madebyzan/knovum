use super::*;

fn job(value: serde_json::Value) -> JobStatusResponse {
    serde_json::from_value(value).expect("valid job payload")
}

#[test]
fn completed_job_prefers_unsigned_content_url() {
    let status = parse_status(job(json!({
        "status": "completed",
        "unsigned_urls": ["https://openrouter.ai/api/v1/videos/id-1/content?index=0"],
        "urls": ["https://cdn.example/video.mp4"],
    })))
    .expect("completed status");
    match status {
        VideoGenerationStatus::Succeeded { video_url } => assert_eq!(
            video_url,
            "https://openrouter.ai/api/v1/videos/id-1/content?index=0"
        ),
        other => panic!("unexpected status: {other:?}"),
    }
}

#[test]
fn completed_job_falls_back_to_signed_url() {
    let status = parse_status(job(json!({
        "status": "completed",
        "urls": ["https://cdn.example/video.mp4"],
    })))
    .expect("completed status");
    assert!(matches!(
        status,
        VideoGenerationStatus::Succeeded { video_url } if video_url == "https://cdn.example/video.mp4"
    ));
}

#[test]
fn failed_job_reads_object_and_string_errors() {
    let from_object = parse_status(job(json!({
        "status": "failed",
        "error": { "code": "provider_error", "message": "upstream refused" },
    })))
    .expect("failed status");
    assert!(matches!(
        from_object,
        VideoGenerationStatus::Failed { error_message } if error_message == "upstream refused"
    ));

    let from_string = parse_status(job(json!({
        "status": "failed",
        "error": "billing limit reached",
    })))
    .expect("failed status");
    assert!(matches!(
        from_string,
        VideoGenerationStatus::Failed { error_message } if error_message == "billing limit reached"
    ));
}

#[test]
fn in_flight_statuses_map_to_polling_states() {
    assert!(matches!(
        parse_status(job(json!({ "status": "pending" }))).expect("status"),
        VideoGenerationStatus::Pending { .. }
    ));
    assert!(matches!(
        parse_status(job(json!({ "status": "in_progress" }))).expect("status"),
        VideoGenerationStatus::Processing { .. }
    ));
}

#[test]
fn unknown_status_is_a_protocol_error() {
    let error = parse_status(job(json!({ "status": "expired" }))).unwrap_err();
    assert_eq!(error.code, "invalid_response");
}
