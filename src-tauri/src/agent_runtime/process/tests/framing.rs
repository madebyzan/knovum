use crate::agent_runtime::process::framing::{self, Frames};
use tokio::io::{AsyncWriteExt, BufReader};

#[tokio::test]
async fn cancelling_observation_does_not_lose_a_partial_frame() {
    let (reader, mut writer) = tokio::io::duplex(1024);
    let mut frames = Frames::new(reader);
    writer
        .write_all(b"{\"jsonrpc\":\"2.0\",\"id\":")
        .await
        .unwrap();
    assert!(
        tokio::time::timeout(std::time::Duration::from_millis(20), frames.next())
            .await
            .is_err()
    );
    writer
        .write_all(b"\"request\",\"result\":null}\n")
        .await
        .unwrap();
    let frame = frames.next().await.unwrap();
    assert_eq!(frame["id"], "request");
    assert!(frame["result"].is_null());
}

#[tokio::test]
async fn eof_and_malformed_frames_are_visible_protocol_failures() {
    for frame in [
        b"{\"jsonrpc\":\"2.0\"}".as_slice(),
        b"{\"jsonrpc\":\"1.0\"}\n".as_slice(),
        b"{\"jsonrpc\":\"2.0\",\"result\":NaN}\n".as_slice(),
    ] {
        let (reader, mut writer) = tokio::io::duplex(1024);
        writer.write_all(frame).await.unwrap();
        writer.shutdown().await.unwrap();
        assert!(framing::read(&mut BufReader::new(reader)).await.is_err());
    }
}
