use super::*;
#[test]
fn cancelled_export_preserves_existing_destination_and_removes_its_partial_file() {
    let work = std::env::temp_dir().join(format!("knovum-export-test-{}", uuid::Uuid::new_v4()));
    fs::create_dir(&work).unwrap();
    let source = work.join("source.wav");
    let destination = work.join("existing.wav");
    fs::write(&source, b"new output").unwrap();
    fs::write(&destination, b"existing output").unwrap();
    let cancelled = CancellationToken::new();
    cancelled.cancel();
    assert_eq!(
        commit(&source, &destination, &cancelled).unwrap_err(),
        "cancelled"
    );
    assert_eq!(fs::read(&destination).unwrap(), b"existing output");
    assert_eq!(fs::read_dir(&work).unwrap().count(), 2);
    commit(&source, &destination, &CancellationToken::new()).unwrap();
    assert_eq!(fs::read(&destination).unwrap(), b"new output");
    fs::remove_dir_all(work).unwrap();
}
