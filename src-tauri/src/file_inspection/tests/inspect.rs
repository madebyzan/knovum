use std::fs;

use super::inspect_file;

#[test]
fn malformed_supported_archive_fails_inspection() {
    let path = std::env::temp_dir().join(format!("{}.zip", uuid::Uuid::new_v4()));
    fs::write(&path, b"not a ZIP archive").expect("write archive fixture");
    let inspection = inspect_file(&path, "broken.zip");
    fs::remove_file(&path).expect("remove archive fixture");
    assert!(inspection.is_err());
}
