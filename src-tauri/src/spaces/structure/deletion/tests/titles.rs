use super::super::list::node_title;

#[test]
fn only_valid_untitled_content_receives_the_display_default() {
    assert_eq!(node_title(r#"{"title":"Known"}"#).unwrap(), "Known");
    assert_eq!(node_title(r#"{}"#).unwrap(), "Untitled");
    assert!(node_title("invalid json").is_err());
    assert!(node_title(r#"{"title":42}"#).is_err());
}
