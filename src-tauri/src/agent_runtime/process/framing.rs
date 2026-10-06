use serde_json::Value;
use tokio::io::{
    AsyncBufRead, AsyncBufReadExt, AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, BufReader,
};

pub(super) const MAX_FRAME_BYTES: usize = 32 * 1024 * 1024;

pub(super) async fn read(reader: &mut (impl AsyncBufRead + Unpin)) -> Result<Value, String> {
    let mut frame = Vec::new();
    reader
        .take((MAX_FRAME_BYTES + 1) as u64)
        .read_until(b'\n', &mut frame)
        .await
        .map_err(|_| "agent-runtime-process-read-failed")?;
    if frame.is_empty() {
        return Err("agent-runtime-process-disconnected".into());
    }
    if frame.len() > MAX_FRAME_BYTES || !frame.ends_with(b"\n") {
        return Err("agent-runtime-frame-limit".into());
    }
    let value: Value = serde_json::from_slice(&frame).map_err(|_| "agent-runtime-invalid-frame")?;
    if !value.is_object() || value["jsonrpc"] != "2.0" {
        return Err("agent-runtime-invalid-frame".into());
    }
    Ok(value)
}

pub(super) async fn write(
    input: &mut (impl AsyncWrite + Unpin),
    value: &Value,
) -> Result<(), String> {
    let mut frame = serde_json::to_vec(value).map_err(|_| "agent-runtime-invalid-frame")?;
    frame.push(b'\n');
    if frame.len() > MAX_FRAME_BYTES {
        return Err("agent-runtime-frame-limit".into());
    }
    input
        .write_all(&frame)
        .await
        .map_err(|_| "agent-runtime-process-write-failed")?;
    input
        .flush()
        .await
        .map_err(|_| "agent-runtime-process-write-failed".into())
}

pub(super) struct Frames {
    receiver: tokio::sync::mpsc::Receiver<Result<Value, String>>,
    reader: tokio::task::JoinHandle<()>,
}

impl Frames {
    pub(super) fn new(output: impl AsyncRead + Unpin + Send + 'static) -> Self {
        let (sender, receiver) = tokio::sync::mpsc::channel(8);
        let reader = tokio::spawn(async move {
            let mut reader = BufReader::new(output);
            loop {
                let frame = read(&mut reader).await;
                let terminal = frame.is_err();
                if sender.send(frame).await.is_err() || terminal {
                    break;
                }
            }
        });
        Self { receiver, reader }
    }

    pub(super) async fn next(&mut self) -> Result<Value, String> {
        self.receiver
            .recv()
            .await
            .ok_or("agent-runtime-process-disconnected")?
    }
}

impl Drop for Frames {
    fn drop(&mut self) {
        self.reader.abort();
    }
}
