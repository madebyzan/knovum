use super::*;

#[tokio::test]
async fn remote_resources_reject_non_public_or_credential_bearing_destinations() {
    for value in [
        "http://example.com/output",
        "https://user:secret@example.com/output",
        "https://127.0.0.1/output",
        "https://[::1]/output",
        "https://localhost/output",
    ] {
        assert!(
            resource_response(
                reqwest::Url::parse(value).unwrap(),
                reqwest::header::HeaderMap::new()
            )
            .await
            .is_err(),
            "{value}"
        );
    }
}

#[test]
fn redirect_credentials_remain_scoped_to_the_original_origin() {
    let origin = reqwest::Url::parse("https://api.example.com/v1/output").unwrap();
    assert!(same_origin(
        &origin,
        &reqwest::Url::parse("https://api.example.com:443/download").unwrap()
    ));
    for value in [
        "https://cdn.example.com/output",
        "http://api.example.com/output",
        "https://api.example.com:8443/output",
    ] {
        assert!(!same_origin(&origin, &reqwest::Url::parse(value).unwrap()));
    }
}

#[test]
fn remote_resources_reject_private_and_special_addresses() {
    for address in [
        "127.0.0.1",
        "10.0.0.1",
        "172.16.0.1",
        "192.168.0.1",
        "169.254.169.254",
        "100.64.0.1",
        "::1",
        "fc00::1",
        "fe80::1",
        "::ffff:127.0.0.1",
        "fec0::1",
    ] {
        assert!(!public_address(address.parse().unwrap()), "{address}");
    }
    assert!(public_address("8.8.8.8".parse().unwrap()));
    assert!(public_address("198.18.0.1".parse().unwrap()));
    assert!(public_address("2606:4700:4700::1111".parse().unwrap()));
}

#[test]
fn ipv6_public_resources_exclude_special_purpose_ranges_at_their_boundaries() {
    for address in [
        "2001::",
        "2001:1::1",
        "2001:1ff:ffff:ffff:ffff:ffff:ffff:ffff",
        "2001:db8::1",
        "2002::1",
        "3fff::",
        "3fff:fff:ffff:ffff:ffff:ffff:ffff:ffff",
        "4000::",
        "::ffff:192.168.0.1",
    ] {
        assert!(!public_address(address.parse().unwrap()), "{address}");
    }
    for address in [
        "2000::",
        "2001:2::1",
        "2001:200::",
        "2003::",
        "3ffe:ffff:ffff:ffff:ffff:ffff:ffff:ffff",
        "3fff:1000::",
        "::ffff:8.8.8.8",
    ] {
        assert!(public_address(address.parse().unwrap()), "{address}");
    }
}
