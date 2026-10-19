use super::require_authority;
use crate::extensions::contracts::{ConnectorAccount, ConnectorAuthorization};
use tokio_util::sync::CancellationToken;

fn account() -> ConnectorAccount {
    ConnectorAccount {
        connector_id: "connector".into(),
        revision: "original".into(),
        resource: "https://example.com/mcp".into(),
        issuer: None,
        scopes: vec![],
        authorization: ConnectorAuthorization::OAuth {
            client_id: None,
            scopes: vec![],
        },
        enabled: true,
        cleanup_pending: false,
    }
}

#[test]
fn old_credential_clients_cannot_rebind_after_reauthorization() {
    let mut account = account();
    let original = account.revision.clone();
    account.revision = "renewed".into();
    assert!(require_authority(&account, &original, None).is_err());
    assert!(require_authority(&account, &original, Some(&CancellationToken::new())).is_err());
    require_authority(&account, &account.revision, None).unwrap();
}

#[test]
fn pending_authorization_does_not_grant_an_old_or_cancelled_client_access() {
    let mut account = account();
    account.enabled = false;
    let current = CancellationToken::new();
    let revoked = CancellationToken::new();
    revoked.cancel();
    require_authority(&account, &account.revision, Some(&current)).unwrap();
    assert!(require_authority(&account, &account.revision, Some(&revoked)).is_err());
    assert!(require_authority(&account, &account.revision, None).is_err());
}

#[test]
fn cleanup_blocks_all_sdk_credential_access_even_with_a_live_authorization() {
    let mut account = account();
    account.cleanup_pending = true;
    assert!(require_authority(&account, &account.revision, None).is_err());
    assert!(
        require_authority(&account, &account.revision, Some(&CancellationToken::new())).is_err()
    );
}
