use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};

use super::bearer_token;

#[test]
fn single_api_key_is_the_bearer_token_verbatim() {
    assert_eq!(bearer_token("sk-abc123").unwrap(), "sk-abc123");
}

#[test]
fn access_secret_pair_signs_a_three_segment_jwt() {
    let token = bearer_token("ak123.sk456").unwrap();
    let segments: Vec<&str> = token.split('.').collect();
    assert_eq!(segments.len(), 3);
    assert_eq!(
        URL_SAFE_NO_PAD.decode(segments[0]).unwrap(),
        br#"{"alg":"HS256","typ":"JWT"}"#
    );
    let claims = String::from_utf8(URL_SAFE_NO_PAD.decode(segments[1]).unwrap()).unwrap();
    assert!(claims.contains(r#""iss":"ak123""#), "{claims}");
    assert!(!segments[2].is_empty());
}

#[test]
fn malformed_key_pair_is_rejected() {
    assert!(bearer_token(".sk456").is_err());
    assert!(bearer_token("ak123.").is_err());
}
