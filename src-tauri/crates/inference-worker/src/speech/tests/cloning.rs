use super::qwen_language;

#[test]
fn qwen_is_conditioned_on_the_language_of_the_line() {
    assert_eq!(qwen_language("zh-CN").unwrap(), "chinese");
    assert_eq!(qwen_language("en").unwrap(), "english");
    assert_eq!(qwen_language("pt_BR").unwrap(), "portuguese");
    assert!(qwen_language("th-TH").is_err());
}
