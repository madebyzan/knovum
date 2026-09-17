use super::protocol::language_name;

#[test]
fn supports_explicit_translation_targets() {
    let supported = [
        "zh-CN", "zh-TW", "en", "ja", "ko", "fr", "de", "es", "pt", "it", "ru", "ar",
    ];
    for target in supported {
        assert!(language_name(target).is_ok(), "target {target}");
    }
    assert!(language_name("auto").is_err());
    assert!(language_name("xyz").is_err());
}
