#[tokio::test]
async fn source_resolution_rejects_credentials_fragments_and_non_https_urls() {
    for url in [
        "http://example.com/plugin.zip",
        "https://user@example.com/plugin.zip",
        "https://user:password@example.com/plugin.zip",
        "https://example.com/plugin.zip#revision",
    ] {
        assert_eq!(
            super::network::resolve_url(url).await.unwrap_err(),
            "package-source-requires-public-https"
        );
    }
}

#[tokio::test]
async fn source_resolution_rejects_private_destinations_before_download_or_git_fetch() {
    for url in [
        "https://127.0.0.1/plugin.zip",
        "https://10.0.0.1/repository.git",
        "https://[::1]/repository.git",
    ] {
        assert_eq!(
            super::network::resolve_url(url).await.unwrap_err(),
            "package-source-requires-public-address"
        );
    }
}
