use super::validate_key;

#[test]
fn validates_product_secret_keys() {
    assert!(validate_key("ai.openai:team-token").is_ok());
    assert!(validate_key("").is_err());
    assert!(validate_key("contains space").is_err());
    assert!(validate_key("../escape").is_err());
}
