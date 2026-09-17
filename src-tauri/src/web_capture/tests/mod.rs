use super::parse_url;

#[test]
fn accepts_http_and_https_urls() {
    assert!(parse_url("https://example.com").is_ok());
    assert!(parse_url("http://127.0.0.1:8080/path").is_ok());
    assert!(parse_url("ftp://example.com").is_err());
    assert!(parse_url("file:///etc/passwd").is_err());
}
