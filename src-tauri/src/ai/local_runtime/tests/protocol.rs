use std::io::Cursor;

use super::{read_frame, BufReader, Protocol, MAX_MESSAGE_BYTES, MAX_REASON_BYTES};

/// A rejection reason names which of the invariants sharing a code was violated.
#[test]
fn worker_rejection_keeps_the_reason_behind_its_code() {
    let rejection = |message: &str, code: &str| {
        Protocol::decode_response::<serde_json::Value>(
            serde_json::json!({
                "jsonrpc": "2.0", "id": "request",
                "error": {"code": -32000, "message": message, "data": {"code": code}}
            }),
            "request",
        )
        .unwrap_err()
    };
    assert_eq!(
        rejection(
            "runtime_mismatch: invalid embedding initialization",
            "runtime_mismatch"
        ),
        "runtime_mismatch: invalid embedding initialization"
    );
    assert_eq!(rejection("cancelled", "cancelled"), "cancelled");
    // A reason that does not refine its code, or that is a payload, is not a diagnostic.
    assert_eq!(
        rejection("unrelated detail", "worker_crashed"),
        "worker_crashed"
    );
    assert_eq!(
        rejection(
            &"runtime_mismatch: ".repeat(MAX_REASON_BYTES),
            "runtime_mismatch"
        ),
        "runtime_mismatch"
    );
}

#[tokio::test]
async fn reads_one_frame_without_consuming_the_next() {
    let mut reader = BufReader::new(Cursor::new(b"{\"value\":1}\nnull\n"));
    assert_eq!(
        read_frame(&mut reader).await.unwrap(),
        serde_json::json!({"value": 1})
    );
    assert!(read_frame(&mut reader).await.unwrap().is_null());
}

#[tokio::test]
async fn frame_limit_includes_the_newline() {
    let mut frame = vec![b' '; MAX_MESSAGE_BYTES];
    frame[..4].copy_from_slice(b"null");
    frame[MAX_MESSAGE_BYTES - 1] = b'\n';
    assert!(read_frame(&mut BufReader::new(Cursor::new(&frame)))
        .await
        .unwrap()
        .is_null());

    frame.insert(MAX_MESSAGE_BYTES - 1, b' ');
    let error = read_frame(&mut BufReader::new(Cursor::new(frame)))
        .await
        .unwrap_err();
    assert_eq!(
        error,
        "invalid_output: worker protocol message exceeds 1 MiB"
    );
}

#[tokio::test]
async fn empty_or_unterminated_stream_reports_worker_crash() {
    for bytes in [b"".as_slice(), b"null".as_slice()] {
        let error = read_frame(&mut BufReader::new(Cursor::new(bytes)))
            .await
            .unwrap_err();
        assert_eq!(error, "worker_crashed: worker closed its protocol stream");
    }
}

#[tokio::test]
async fn invalid_utf8_reports_invalid_output() {
    let error = read_frame(&mut BufReader::new(Cursor::new(b"\"\xff\"\n")))
        .await
        .unwrap_err();
    assert!(error.starts_with("invalid_output:"));
}
