use std::io::Cursor;

use super::protocol::{self, Request, Response};
use crate::documents::generation::{document::DocumentSource, DocumentFormat};

fn request() -> Vec<u8> {
    protocol::encode_request(DocumentFormat::Text, DocumentSource::default()).unwrap()
}

#[test]
fn worker_uses_one_strict_versioned_frame() {
    let bytes = request();
    let accepted = protocol::read_request(&mut Cursor::new(bytes.clone())).unwrap();
    assert_eq!(accepted.version, protocol::VERSION);
    assert!(matches!(accepted.format, DocumentFormat::Text));
    let mut trailing = bytes;
    trailing.push(0);
    assert!(protocol::read_request(&mut Cursor::new(trailing)).is_err());
    for payload in [
        br#"{"version":2,"format":"text","source":{"title":"","markdown":"","citations":[]}}"#.as_slice(),
        br#"{"version":1,"format":"text","source":{"title":"","markdown":"","citations":[],"path":"secret"}}"#,
        br#"{"version":1,"format":"html","source":{"title":"","markdown":"","citations":[]}}"#,
    ] {
        let mut frame = Vec::new();
        protocol::write_frame(&mut frame, payload).unwrap();
        assert!(protocol::read_request(&mut Cursor::new(frame)).is_err());
    }
}

#[test]
fn worker_rejects_oversized_input_before_reading_payload() {
    let bytes = (protocol::MAX_REQUEST_BYTES + 1).to_be_bytes();
    assert!(protocol::read_request(&mut Cursor::new(bytes)).is_err());
    assert!(protocol::encode_request(
        DocumentFormat::Text,
        DocumentSource {
            markdown: "a".repeat(4 * 1024 * 1024 + 1),
            ..Default::default()
        }
    )
    .is_err());
}

#[test]
fn worker_render_failure_is_a_typed_response_without_output_bytes() {
    let source = DocumentSource {
        markdown: "<script>bad</script>".into(),
        ..Default::default()
    };
    let request = protocol::encode_request(DocumentFormat::Text, source).unwrap();
    let mut output = Vec::new();
    super::serve(&mut Cursor::new(request), &mut output).unwrap();
    let mut reader = Cursor::new(output);
    let header = protocol::read_frame(&mut reader, protocol::MAX_HEADER_BYTES).unwrap();
    let response: Response = serde_json::from_slice(&header).unwrap();
    assert!(matches!(response.outcome, protocol::Outcome::Failed { .. }));
    assert_eq!(reader.position() as usize, reader.get_ref().len());
}

#[test]
fn worker_returns_exact_binary_content_and_consumes_no_space_metadata() {
    let source = DocumentSource {
        title: "标题".into(),
        markdown: "内容".into(),
        ..Default::default()
    };
    let expected = DocumentFormat::Text.render(&source).unwrap();
    let request = protocol::encode_request(DocumentFormat::Text, source).unwrap();
    let mut output = Vec::new();
    super::serve(&mut Cursor::new(request), &mut output).unwrap();
    let mut reader = Cursor::new(output);
    let header = protocol::read_frame(&mut reader, protocol::MAX_HEADER_BYTES).unwrap();
    let response: Response = serde_json::from_slice(&header).unwrap();
    assert_eq!(response.version, protocol::VERSION);
    assert!(
        matches!(response.outcome, protocol::Outcome::Rendered { length } if length as usize == expected.len())
    );
    assert_eq!(
        &reader.get_ref()[reader.position() as usize..],
        expected.as_slice()
    );
}

#[test]
fn request_source_fields_are_required() {
    assert!(serde_json::from_str::<Request>(
        r#"{"version":1,"format":"text","source":{"title":""}}"#
    )
    .is_err());
}
