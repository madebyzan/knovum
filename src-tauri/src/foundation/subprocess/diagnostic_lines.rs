use tokio::io::{AsyncRead, AsyncReadExt};

pub(crate) async fn consume(
    mut stream: impl AsyncRead + Unpin,
    max_line_bytes: usize,
    mut inspect: impl FnMut(&[u8]),
) -> std::io::Result<()> {
    let mut chunk = [0_u8; 4096];
    let mut line = Vec::new();
    let mut oversized = false;
    loop {
        let size = stream.read(&mut chunk).await?;
        if size == 0 {
            break;
        }
        for byte in &chunk[..size] {
            if *byte == b'\n' {
                if !oversized {
                    inspect(&line);
                }
                line.clear();
                oversized = false;
            } else if !oversized {
                if line.len() < max_line_bytes {
                    line.push(*byte);
                } else {
                    line.clear();
                    oversized = true;
                }
            }
        }
    }
    if !oversized && !line.is_empty() {
        inspect(&line);
    }
    Ok(())
}
