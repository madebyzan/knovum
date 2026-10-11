use std::io::Cursor;

use super::super::protocol::read_frame;

#[test]
fn rejects_unterminated_and_oversized_frames_before_parsing() {
    assert!(read_frame(&mut Cursor::new(b"{}"), 8).is_err());
    assert!(read_frame(&mut Cursor::new(b"123456789\n"), 8).is_err());
    assert_eq!(
        read_frame(&mut Cursor::new(b"{}\n"), 8).unwrap().unwrap(),
        serde_json::json!({})
    );
    assert!(read_frame(&mut Cursor::new(b""), 8).unwrap().is_none());
}

#[test]
fn bounded_reader_preserves_the_next_complete_frame() {
    let mut reader = Cursor::new(b"{}\n[]\n");
    assert_eq!(
        read_frame(&mut reader, 8).unwrap().unwrap(),
        serde_json::json!({})
    );
    assert_eq!(
        read_frame(&mut reader, 8).unwrap().unwrap(),
        serde_json::json!([])
    );
}
