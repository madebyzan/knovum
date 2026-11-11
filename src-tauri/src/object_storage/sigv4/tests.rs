use chrono::{TimeZone, Utc};

use super::{authorization, ensure_signable_url, signing_key};
use crate::object_storage::settings::Credentials;

fn credentials() -> Credentials {
    Credentials {
        access_key_id: "AKIAIOSFODNN7EXAMPLE".to_owned(),
        secret_access_key: "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY".to_owned(),
    }
}

#[test]
fn signing_key_matches_the_documented_aws_derivation() {
    // AWS General Reference signing-key derivation example:
    // secret wJalrXUtnFEMI/K7MDENG+bPxRfiCYEXAMPLEKEY, 20120215/us-east-1/iam,
    // kSigning f4780e2d9f65fa895f9c67b32ce1baf0b0d8a43505a000a1a9e090d414db404d.
    let key = signing_key(
        "wJalrXUtnFEMI/K7MDENG+bPxRfiCYEXAMPLEKEY",
        "20120215",
        "us-east-1",
        "iam",
    );
    assert_eq!(
        hex::encode(ring::hmac::sign(&key, b"").as_ref()),
        "de9dfa831dc0cfb138762217e8afc1fd3e66df076b21d2bde5ff858bfc2e2575"
    );
}

#[test]
fn authorization_signs_the_canonical_put_shape() {
    let url = "https://cos.ap-guangzhou.myqcloud.com/knovum-bucket/knovum/2026/11/abcd.png"
        .parse()
        .unwrap();
    let signed = authorization(
        "PUT",
        &url,
        "ap-guangzhou",
        &credentials(),
        Some("image/png"),
        b"pixels",
        Utc.with_ymd_and_hms(2026, 11, 11, 0, 0, 0).unwrap(),
    )
    .unwrap();
    assert_eq!(signed.amz_date, "20261111T000000Z");
    assert!(signed.authorization.starts_with(
        "AWS4-HMAC-SHA256 Credential=AKIAIOSFODNN7EXAMPLE/20261111/ap-guangzhou/s3/aws4_request, \
         SignedHeaders=content-type;host;x-amz-content-sha256;x-amz-date, Signature="
    ));
    // SHA-256 of the payload travels in x-amz-content-sha256.
    use sha2::Digest;
    assert_eq!(
        signed.payload_hash,
        hex::encode(sha2::Sha256::digest(b"pixels"))
    );
}

#[test]
fn authorization_without_content_type_omits_it_from_signed_headers() {
    let url = "https://bucket.oss-cn-hangzhou.aliyuncs.com/key"
        .parse()
        .unwrap();
    let signed = authorization(
        "HEAD",
        &url,
        "oss-cn-hangzhou",
        &credentials(),
        None,
        b"",
        Utc.with_ymd_and_hms(2026, 11, 11, 0, 0, 0).unwrap(),
    )
    .unwrap();
    assert!(signed
        .authorization
        .contains("SignedHeaders=host;x-amz-content-sha256;x-amz-date"));
}

#[test]
fn signable_urls_reject_queries_and_unsafe_paths() {
    assert!(ensure_signable_url(&"https://host.example/bucket/key".parse().unwrap()).is_ok());
    assert!(ensure_signable_url(&"https://host.example/bucket/key?acl".parse().unwrap()).is_err());
    assert!(ensure_signable_url(&"https://host.example/bucket/ke%20y".parse().unwrap()).is_err());
}
