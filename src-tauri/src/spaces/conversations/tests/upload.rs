use std::path::Path;

use super::{classify, enforce_size, UploadKind};

#[test]
fn upload_validation_returns_explicit_command_codes() {
    assert_eq!(
        classify(Path::new("attachment.exe")).err().unwrap().code,
        "conversation_upload_unsupported"
    );
    assert_eq!(
        enforce_size(UploadKind::Text, 0).err().unwrap().code,
        "conversation_upload_size"
    );
}
