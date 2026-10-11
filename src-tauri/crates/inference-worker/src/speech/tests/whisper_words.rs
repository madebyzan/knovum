use super::{append, finish};

#[test]
fn english_subword_tokens_share_one_word_timeline() {
    let mut words = Vec::new();
    append(&mut words, " know", 10, 20, 0.9, "en");
    append(&mut words, "ledge", 20, 30, 0.8, "en");
    assert_eq!(words.len(), 1);
    assert_eq!(words[0].text, " knowledge");
    assert_eq!((words[0].start_ms, words[0].end_ms), (10, 30));
}

#[test]
fn published_words_are_bounded_by_their_segment_and_have_positive_duration() {
    let mut words = Vec::new();
    append(&mut words, " 知识", 0, 50, 0.9, "zh");
    append(&mut words, "运行", 50, 110, 0.9, "zh");
    append(&mut words, "。", 110, 110, 0.9, "zh");
    finish(&mut words, 10, 100);
    assert_eq!(words.len(), 2);
    assert_eq!(words[0].text, "知识");
    assert_eq!((words[0].start_ms, words[0].end_ms), (10, 50));
    assert_eq!((words[1].start_ms, words[1].end_ms), (50, 100));
}

#[test]
fn languages_without_spaces_keep_complete_unicode_token_timelines() {
    for language in ["zh", "ja", "th", "lo", "my", "yue"] {
        let mut words = Vec::new();
        append(&mut words, "知识", 10, 20, 0.9, language);
        append(&mut words, "保存", 20, 30, 0.8, language);
        assert_eq!(words.len(), 2, "language: {language}");
        assert_eq!((words[1].start_ms, words[1].end_ms), (20, 30));
    }
}
