use super::{language_code, language_label, LANGUAGES};

#[test]
fn every_language_round_trips_between_its_code_and_label() {
    for (code, label) in LANGUAGES {
        assert_eq!(language_label(code), Some(label));
        assert_eq!(language_code(label), Some(code));
    }
}

#[test]
fn language_labels_are_matched_without_regard_to_case() {
    assert_eq!(language_code("chinese"), Some("zh"));
    assert_eq!(language_code("CANTONESE"), Some("yue"));
}

#[test]
fn an_unknown_language_is_rejected_rather_than_guessed() {
    assert_eq!(language_label("xx"), None);
    assert_eq!(language_code("Klingon"), None);
    assert_eq!(language_code(""), None);
}
