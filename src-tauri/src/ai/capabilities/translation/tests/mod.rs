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

#[test]
fn dubbing_input_embeds_timing_constraints_universally() {
    for lang in ["ja", "de", "es", "fr", "zh-CN", "en"] {
        let input = super::dubbing_input("Hello world".into(), lang.into(), Some(2400)).unwrap();
        assert!(input.system_prompt.contains("2.40s (2400ms)"));
        assert!(input
            .system_prompt
            .contains("expert dubbing localization adapter"));
        assert!(!input.system_prompt.contains("Chinese characters"));
        assert!(!input.system_prompt.contains("English words"));
    }
}
