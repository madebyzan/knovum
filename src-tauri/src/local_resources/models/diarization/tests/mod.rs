use super::*;
use sha2::{Digest, Sha256};

#[test]
fn owned_config_and_file_inventory_match_pinned_digests_and_size() {
    assert_eq!(
        hex::encode(Sha256::digest(manifest::CONFIG.as_bytes())),
        manifest::CONFIG_SHA256
    );
    assert_eq!(
        manifest::FILES.iter().map(|file| file.size).sum::<u64>(),
        manifest::TOTAL_BYTES
    );
    assert_eq!(manifest::REVISION.len(), 40);
    for file in manifest::FILES {
        assert_eq!(file.sha256.len(), 64);
        assert!(file.sha256.bytes().all(|byte| byte.is_ascii_hexdigit()));
    }
}

#[tokio::test]
async fn invalid_access_tokens_are_rejected_before_secret_store_access() {
    for token in [
        String::new(),
        " ".into(),
        "invalid\nheader".into(),
        "x".repeat(4097),
    ] {
        let error = commands::set_diarization_access_token(Some(token))
            .await
            .unwrap_err();
        assert_eq!(error.code, "invalid_request");
        assert!(!error.message.contains("header"));
    }
}
