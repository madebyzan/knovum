use std::io::Cursor;

use super::{read_frame, BufReader, MAX_MESSAGE_BYTES};

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
