use super::{protocol, read_response};

fn response(header: serde_json::Value, bytes: &[u8]) -> Vec<u8> {
    let mut frame = vec![];
    protocol::write_frame(&mut frame, &serde_json::to_vec(&header).unwrap()).unwrap();
    frame.extend_from_slice(bytes);
    frame
}

#[tokio::test]
async fn accepts_exact_binary_content_and_requires_eof() {
    let header = serde_json::json!({"version": protocol::VERSION, "outcome": {"status": "rendered", "length": 4}});
    let binary = [0, 255, 13, 10];
    assert_eq!(
        read_response(response(header.clone(), &binary).as_slice())
            .await
            .unwrap()
            .unwrap(),
        binary
    );
    let mut trailing = response(header, &binary);
    trailing.push(0);
    assert_eq!(
        read_response(trailing.as_slice()).await.unwrap_err().code,
        "document_worker_failed"
    );
}

#[tokio::test]
async fn rejects_unknown_fields_versions_and_oversized_output_before_reading_it() {
    for header in [
        serde_json::json!({"version": protocol::VERSION + 1, "outcome": {"status": "rendered", "length": 0}}),
        serde_json::json!({"version": protocol::VERSION, "outcome": {"status": "rendered", "length": 0}, "unexpected": true}),
        serde_json::json!({"version": protocol::VERSION, "outcome": {"status": "rendered", "length": 0, "unexpected": true}}),
        serde_json::json!({"version": protocol::VERSION, "outcome": {"status": "rendered", "length": protocol::MAX_OUTPUT_BYTES + 1}}),
    ] {
        assert_eq!(
            read_response(response(header, &[]).as_slice())
                .await
                .unwrap_err()
                .code,
            "document_worker_failed"
        );
    }
}

#[tokio::test]
async fn rejects_truncated_binary_and_unbounded_header_frames() {
    let header = serde_json::json!({"version": protocol::VERSION, "outcome": {"status": "rendered", "length": 2}});
    assert!(read_response(response(header, &[0]).as_slice())
        .await
        .is_err());
    let oversized = (protocol::MAX_HEADER_BYTES + 1).to_be_bytes();
    assert!(read_response(oversized.as_slice()).await.is_err());
}

#[tokio::test]
async fn rendering_failure_remains_distinct_from_protocol_failure() {
    let header = serde_json::json!({"version": protocol::VERSION, "outcome": {"status": "failed", "message": "Unsupported document content"}});
    let error = read_response(response(header, &[]).as_slice())
        .await
        .unwrap()
        .unwrap_err();
    assert_eq!(error.code, "document_render_failed");
    assert_eq!(error.message, "Unsupported document content");
}
