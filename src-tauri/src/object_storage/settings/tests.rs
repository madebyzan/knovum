use super::{validate, ObjectStorageSettings};

fn valid() -> ObjectStorageSettings {
    ObjectStorageSettings {
        endpoint: "https://cos.ap-guangzhou.myqcloud.com".to_owned(),
        region: "ap-guangzhou".to_owned(),
        bucket: "knovum-1250000000".to_owned(),
        public_base_url: None,
    }
}

#[test]
fn accepts_a_minimal_s3_compatible_configuration() {
    assert!(validate(&valid()).is_ok());
}

#[test]
fn rejects_non_https_endpoints() {
    let mut settings = valid();
    settings.endpoint = "http://cos.ap-guangzhou.myqcloud.com".to_owned();
    assert!(validate(&settings).is_err());
    settings.endpoint = "cos.ap-guangzhou.myqcloud.com".to_owned();
    assert!(validate(&settings).is_err());
}

#[test]
fn rejects_urls_with_embedded_credentials() {
    let mut settings = valid();
    settings.endpoint = "https://user:pass@cos.ap-guangzhou.myqcloud.com".to_owned();
    assert!(validate(&settings).is_err());
}

#[test]
fn rejects_invalid_buckets_and_regions() {
    let mut settings = valid();
    settings.bucket = "AB".to_owned();
    assert!(validate(&settings).is_err());
    settings.bucket = "-leading-dash-bucket".to_owned();
    assert!(validate(&settings).is_err());
    settings.bucket = "knovum-1250000000".to_owned();
    settings.region = " ".to_owned();
    assert!(validate(&settings).is_err());
}

#[test]
fn public_url_prefers_the_configured_base() {
    let settings = valid();
    assert_eq!(
        settings.public_url("knovum/2026/11/abc.png"),
        "https://cos.ap-guangzhou.myqcloud.com/knovum-1250000000/knovum/2026/11/abc.png"
    );
    let with_base = ObjectStorageSettings {
        public_base_url: Some("https://cdn.example.com".to_owned()),
        ..valid()
    };
    assert_eq!(
        with_base.public_url("knovum/2026/11/abc.png"),
        "https://cdn.example.com/knovum/2026/11/abc.png"
    );
    let with_trailing_slash = ObjectStorageSettings {
        public_base_url: Some("https://cdn.example.com/".to_owned()),
        ..valid()
    };
    assert_eq!(
        with_trailing_slash.public_url("knovum/2026/11/abc.png"),
        "https://cdn.example.com/knovum/2026/11/abc.png"
    );
}
