use std::sync::mpsc;

use super::{spawn_reader, StreamLine, MAX_OUTPUT_LINE_BYTES};

#[test]
fn reader_rejects_a_line_above_the_output_budget() {
    let (sender, receiver) = mpsc::sync_channel(2);
    let reader = spawn_reader(
        std::io::Cursor::new(vec![b'x'; MAX_OUTPUT_LINE_BYTES + 1]),
        sender,
        StreamLine::Stderr,
    );
    assert!(matches!(receiver.recv(), Ok(StreamLine::Error(_))));
    reader.join().expect("reader thread must finish");
}
