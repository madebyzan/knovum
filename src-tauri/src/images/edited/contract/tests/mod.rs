use super::ImageEdit;

#[test]
fn image_edit_contract_rejects_obsolete_version() {
    let mut edit = ImageEdit::identity();
    assert_eq!(edit.version, 1);
    assert!(edit.validate().is_ok());
    for unsupported in [0, 2, 5, 6] {
        edit.version = unsupported;
        assert_eq!(edit.validate().unwrap_err(), "imageEdit.version must be 1");
    }
}
