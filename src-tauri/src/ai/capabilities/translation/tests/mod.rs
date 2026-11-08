use super::protocol::language_name;
use super::speech_rate::{estimate_duration_ms, requires_reduction, syllables_per_second};

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

#[test]
fn speech_rate_table_covers_every_translation_target() {
    for target in [
        "zh-CN", "zh-TW", "en", "en-US", "ja", "ko", "fr", "de", "es", "pt-BR", "it", "ru", "ar",
    ] {
        assert!(syllables_per_second(target).is_some(), "target {target}");
    }
    assert!(syllables_per_second("auto").is_none());
    assert!(syllables_per_second("xyz").is_none());
}

#[test]
fn duration_estimates_follow_script_conventions() {
    // One syllable per CJK / hangul / kana character.
    assert_eq!(estimate_duration_ms("你好世界", "zh-CN"), Some(769)); // 4 / 5.2
    assert_eq!(estimate_duration_ms("こんにちは", "ja"), Some(641)); // 5 / 7.8
    assert_eq!(estimate_duration_ms("안녕하세요", "ko"), Some(781)); // 5 / 6.4
                                                                     // One syllable per maximal vowel group in alphabetic words.
    assert_eq!(estimate_duration_ms("beautiful", "en"), Some(500)); // 3 / 6.0
    assert_eq!(estimate_duration_ms("yellow", "en"), Some(333)); // 2 / 6.0
    assert_eq!(estimate_duration_ms("привет", "ru"), Some(290)); // 2 / 6.9
    assert_eq!(estimate_duration_ms("سلام", "ar"), Some(167)); // 1 / 6.0
                                                              // Written digits each cost about one syllable when spoken.
    assert_eq!(estimate_duration_ms("3.14", "en"), Some(500)); // 3 / 6.0
    assert_eq!(estimate_duration_ms("", "en"), Some(0));
    assert_eq!(estimate_duration_ms("hello", "auto"), None);
}

#[test]
fn reduction_only_triggers_on_clear_overruns() {
    assert!(!requires_reduction(1200, 1000));
    assert!(requires_reduction(1201, 1000));
    assert!(!requires_reduction(0, 1000));
    assert!(requires_reduction(2401, 2000));
}

#[test]
fn dubbing_reduction_input_carries_window_and_previous_adaptation() {
    let input =
        super::dubbing_reduction_input("源文本".into(), "zh-CN".into(), 1000, "之前的译文", 2400)
            .unwrap();
    assert!(input.system_prompt.contains("2.40s"));
    assert!(input.system_prompt.contains("1.00s"));
    assert!(input.system_prompt.contains("58%"));
    assert_eq!(input.instruction, "Previous adaptation:\n之前的译文");
    assert_eq!(input.text, "源文本");
    assert!(super::dubbing_reduction_input("t".into(), "auto".into(), 1000, "p", 2400).is_err());
}
