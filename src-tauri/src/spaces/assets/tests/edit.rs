use serde_json::json;

use super::{atomic_replace_file, ensure_pdf_node, validate_pdf_bytes};

#[test]
fn admits_only_pdf_file_nodes() {
    assert!(ensure_pdf_node(&json!({"kind": "file", "fileName": "Brief.PDF"})).is_ok());
    assert!(ensure_pdf_node(&json!({"kind": "code", "fileName": "app.ts"})).is_err());
    assert!(ensure_pdf_node(&json!({"kind": "file", "fileName": "sheet.xlsx"})).is_err());
}

#[test]
fn validates_pdf_header_and_bounds() {
    assert!(validate_pdf_bytes(b"%PDF-1.7\n%%EOF").is_ok());
    assert!(validate_pdf_bytes(b"").is_err());
    assert!(validate_pdf_bytes(b"not a pdf").is_err());
}

#[test]
fn atomically_replaces_existing_file_without_leaving_temp_files() {
    let dir = std::env::temp_dir().join(format!("knovum-edit-test-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&dir).unwrap();
    let target = dir.join("sample.pdf");
    std::fs::write(&target, b"%PDF-1.4\nold").unwrap();
    atomic_replace_file(&target, b"%PDF-1.7\nnew").unwrap();
    assert_eq!(std::fs::read(&target).unwrap(), b"%PDF-1.7\nnew");
    let entries: Vec<_> = std::fs::read_dir(&dir).unwrap().collect();
    assert_eq!(entries.len(), 1);
    let _ = std::fs::remove_dir_all(&dir);
}
