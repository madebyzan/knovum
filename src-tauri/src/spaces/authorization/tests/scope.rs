use super::*;

#[test]
fn local_user_grant_owns_principal_scope_and_named_ttl() {
    let grant = local_user_grant(
        "grant".into(),
        "space".into(),
        vec!["create-note".into()],
        42,
    );
    assert_eq!(grant.principal.kind, PrincipalKind::User);
    assert_eq!(grant.principal.id, "local-user");
    assert_eq!(grant.read_space_ids, vec!["space".to_string()]);
    assert_eq!(grant.write_space_ids, vec!["space".to_string()]);
    assert_eq!(grant.action_ids, vec!["create-note".to_string()]);
    assert_eq!(grant.valid_until, 42 + LOCAL_USER_GRANT_TTL_MS);
}

#[test]
fn external_agent_grant_preserves_the_explicit_action_set() {
    let grant = external_agent_grant(
        "grant".into(),
        "space".into(),
        "agent".into(),
        vec!["read-node".into(), "create-note".into()],
        42,
    );
    assert_eq!(grant.principal.kind, PrincipalKind::ExternalAgent);
    assert_eq!(grant.principal.id, "agent");
    assert_eq!(
        grant.action_ids,
        vec!["read-node".to_string(), "create-note".to_string()]
    );
    assert_eq!(grant.valid_until, 42 + EXTERNAL_AGENT_GRANT_TTL_MS);
}
