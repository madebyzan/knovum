use super::super::import::{validate_source_ids, JsonCanvasNodeImport};

#[test]
fn rejects_duplicate_source_ids_before_persistence() {
    let source = r#"{"sourceId":"n","node":{"id":"id","kind":"note","initialX":0,"initialY":0,"data":{},"sourceChunks":[]}}"#;
    let first: JsonCanvasNodeImport = serde_json::from_str(source).unwrap();
    let second: JsonCanvasNodeImport = serde_json::from_str(source).unwrap();
    assert!(validate_source_ids(&[first, second]).is_err());
}
