use super::*;
#[test]
fn creates_text_and_link_payloads_without_structure_fields() {
    for (source, expected) in [
        (
            json!({"id":"text","type":"text","x":0,"y":0,"width":200,"height":120,"text":"Evidence"}),
            json!({"kind":"note","plainText":"Evidence"}),
        ),
        (
            json!({"id":"link","type":"link","x":0,"y":0,"width":200,"height":120,"url":"https://example.com/research"}),
            json!({"kind":"website","url":"https://example.com/research"}),
        ),
    ] {
        let node = serde_json::from_value(source).unwrap();
        let actual = payload(&node, "Note", None).unwrap();
        for (key, value) in expected.as_object().unwrap() {
            assert_eq!(&actual[key], value);
        }
        assert!(actual.get("sectionId").is_none());
        assert!(actual.get("locked").is_none());
    }
}
#[test]
fn website_import_accepts_host_input_and_rejects_unsafe_schemes() {
    assert_eq!(
        website_url("www.example.com/path").unwrap().as_str(),
        "https://www.example.com/path"
    );
    assert_eq!(
        website_url("example.com:8080/path").unwrap().as_str(),
        "https://example.com:8080/path"
    );
    assert!(website_url("example .com").is_err());
    let node = serde_json::from_value(json!({"id":"link","type":"link","x":0,"y":0,"width":200,"height":120,"url":"file:///tmp/secret"})).unwrap();
    assert!(payload(&node, "Note", None).is_err());
}
