use super::protocol::language_name;
use super::speech_rate::{estimate_duration_ms, requires_reduction, syllables_per_second};
use super::{
    dubbing_batch_input, dubbing_input, dubbing_reduction_input, parse_batch_output, BatchLineInput,
};

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
        let input = dubbing_input("Hello world".into(), lang.into(), Some(2400), &[]).unwrap();
        assert!(input.system_prompt.contains("2.40s (2400ms)"));
        assert!(input
            .system_prompt
            .contains("expert dubbing localization adapter"));
        assert!(!input.system_prompt.contains("Chinese characters"));
        assert!(!input.system_prompt.contains("English words"));
    }
}

#[test]
fn dubbing_input_embeds_term_preservation() {
    let terms = vec!["Knovum".into(), "DeepSeek".into()];
    let input = dubbing_input("Hello world".into(), "zh-CN".into(), Some(2400), &terms).unwrap();
    assert!(input.system_prompt.contains(
        "Do NOT translate, transliterate, or alter the following proper nouns and terms"
    ));
    assert!(input.system_prompt.contains("Knovum, DeepSeek"));
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
    let terms = vec!["BrandX".into()];
    let input = dubbing_reduction_input(
        "源文本".into(),
        "zh-CN".into(),
        1000,
        "之前的译文",
        2400,
        &terms,
    )
    .unwrap();
    assert!(input.system_prompt.contains("2.40s"));
    assert!(input.system_prompt.contains("1.00s"));
    assert!(input.system_prompt.contains("58%"));
    assert!(input.system_prompt.contains("BrandX"));
    assert_eq!(input.instruction, "Previous adaptation:\n之前的译文");
    assert_eq!(input.text, "源文本");
    assert!(dubbing_reduction_input("t".into(), "auto".into(), 1000, "p", 2400, &[]).is_err());
}

#[test]
fn batch_translation_input_and_parser_work_with_fences_and_pure_json() {
    let lines = vec![
        BatchLineInput {
            id: 0,
            text: "Hello world".into(),
            duration_ms: Some(2000),
        },
        BatchLineInput {
            id: 1,
            text: "Welcome to Knovum".into(),
            duration_ms: Some(3000),
        },
    ];
    let terms = vec!["Knovum".into()];
    let input = dubbing_batch_input(&lines, "zh-CN".into(), &terms).unwrap();
    assert!(input.system_prompt.contains("Knovum"));
    assert!(input.text.contains("Hello world"));

    let markdown_json = "```json\n[\n  {\"id\": 0, \"translation\": \"你好世界\"},\n  {\"id\": 1, \"translation\": \"欢迎来到 Knovum\"}\n]\n```";
    let parsed = parse_batch_output(markdown_json).unwrap();
    assert_eq!(parsed.get(&0).unwrap(), "你好世界");
    assert_eq!(parsed.get(&1).unwrap(), "欢迎来到 Knovum");

    let pure_json = "[{\"id\":0,\"translation\":\"First\"}]";
    let parsed_pure = parse_batch_output(pure_json).unwrap();
    assert_eq!(parsed_pure.get(&0).unwrap(), "First");
}
