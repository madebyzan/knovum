
use super::ImageEdit;

#[test]
fn image_edit_contract_rejects_obsolete_version() {
    let mut edit = ImageEdit::identity();
    assert!(edit.validate().is_ok());
    edit.version = 5;
    assert_eq!(edit.validate().unwrap_err(), "imageEdit.version must be 6");
}
