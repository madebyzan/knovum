use super::*;
use crate::app_error::AppError;

fn fixture() -> std::path::PathBuf {
    let path = std::env::temp_dir().join(format!("knovum-revision-test-{}", uuid::Uuid::new_v4()));
    fs::write(&path, vec![0x5a; BUFFER_BYTES * 2]).expect("revision fixture");
    path
}

#[test]
fn cancellable_revision_matches_plain_revision() {
    let path = fixture();
    let plain = content_revision(&path).expect("plain revision");
    let cancellable =
        content_revision_cancellable::<AppError>(&path, || Ok(())).expect("cancellable revision");
    fs::remove_file(path).expect("remove revision fixture");
    assert_eq!(cancellable, plain);
}

#[test]
fn cancellable_revision_preserves_typed_cancellation() {
    let path = fixture();
    let mut checks = 0;
    let error = content_revision_cancellable::<AppError>(&path, || {
        checks += 1;
        if checks >= 3 {
            Err(AppError::cancelled("hash cancelled"))
        } else {
            Ok(())
        }
    })
    .expect_err("revision must cancel between chunks");
    fs::remove_file(path).expect("remove revision fixture");
    assert_eq!(error.code, "cancelled");
}
