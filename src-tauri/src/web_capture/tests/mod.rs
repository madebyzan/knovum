use super::{decode_document, parse_url, validate_derived_url};
use crate::foundation::public_network::public_address;

#[test]
fn accepts_http_and_https_urls() {
    assert!(parse_url("https://example.com").is_ok());
    assert!(parse_url("http://127.0.0.1:8080/path").is_ok());
    assert!(parse_url("ftp://example.com").is_err());
    assert!(parse_url("file:///etc/passwd").is_err());
}

#[test]
fn derived_images_require_credential_free_web_urls() {
    assert!(validate_derived_url(&parse_url("https://example.com/image.png").unwrap()).is_ok());
    assert!(validate_derived_url(&parse_url("http://example.com/favicon.ico").unwrap()).is_ok());
    assert!(
        validate_derived_url(&parse_url("https://user@example.com/image.png").unwrap()).is_err()
    );
    assert!(
        validate_derived_url(&parse_url("https://user:secret@example.com/image.png").unwrap())
            .is_err()
    );
}

#[test]
fn derived_images_reject_private_and_special_addresses() {
    for address in [
        "127.0.0.1",
        "10.0.0.1",
        "172.16.0.1",
        "192.168.0.1",
        "169.254.169.254",
        "100.64.0.1",
        "0.0.0.0",
        "224.0.0.1",
        "240.0.0.1",
        "::1",
        "fc00::1",
        "fe80::1",
        "::ffff:127.0.0.1",
        "2002::1",
    ] {
        assert!(!public_address(address.parse().unwrap()), "{address}");
    }
    assert!(public_address("8.8.8.8".parse().unwrap()));
    assert!(public_address("2606:4700:4700::1111".parse().unwrap()));
}

#[tokio::test]
async fn derived_images_reject_loopback_before_connecting() {
    let error = super::capture_derived_image("http://127.0.0.1:9/favicon.ico")
        .await
        .unwrap_err();
    assert!(error.contains("public addresses"), "{error}");
}

#[test]
fn decodes_declared_legacy_chinese_without_replacement() {
    let media_type = "text/html; charset=gbk".parse().unwrap();
    let (bytes, _, had_errors) = encoding_rs::GBK.encode("中文正文");
    assert!(!had_errors);
    assert_eq!(
        decode_document(Some(&media_type), &bytes).unwrap(),
        "中文正文"
    );
}

#[test]
fn rejects_invalid_or_unsupported_document_encoding() {
    let utf8 = "text/plain; charset=utf-8".parse().unwrap();
    assert_eq!(
        decode_document(Some(&utf8), &[0xff]).unwrap_err(),
        "Website document contains invalid encoded text"
    );

    let unsupported = "text/plain; charset=x-knovum-unknown".parse().unwrap();
    assert_eq!(
        decode_document(Some(&unsupported), b"text").unwrap_err(),
        "Website document charset is not supported"
    );
}

#[test]
fn bom_selects_the_document_encoding_before_the_http_default() {
    let media_type = "text/plain".parse().unwrap();
    let bytes = [0xff, 0xfe, b'A', 0x00];
    assert_eq!(decode_document(Some(&media_type), &bytes).unwrap(), "A");
}
