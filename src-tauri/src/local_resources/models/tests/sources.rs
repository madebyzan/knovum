use super::super::{catalog, sources};

#[test]
fn endpoints_preserve_the_same_pinned_artifact() {
    let descriptor = catalog::FASTER_WHISPER_WEIGHTS;
    let urls = sources::urls(
        descriptor,
        &["https://huggingface.co", "https://hf-mirror.com/"],
    )
    .unwrap();
    assert_eq!(urls[0].path(), urls[1].path());
    assert!(urls[0].path().contains(descriptor.revision));
    assert!(urls[0].path().ends_with(descriptor.file));
    assert_eq!(urls[1].host_str(), Some("hf-mirror.com"));
    assert_eq!(urls[1].query(), Some("download=true"));
    let custom = sources::urls(descriptor, &["https://models.example/hub/"]).unwrap();
    assert_eq!(custom.len(), 1);
    assert!(custom[0].path().starts_with("/hub/Systran/"));
}

#[test]
fn unsafe_or_malformed_endpoints_are_rejected() {
    for endpoint in [
        "",
        "http://example.com",
        "https://user:secret@example.com",
        "https://example.com?token=x",
        "https://example.com/#fragment",
    ] {
        assert!(sources::urls(catalog::FASTER_WHISPER_WEIGHTS, &[endpoint]).is_err());
    }
}
