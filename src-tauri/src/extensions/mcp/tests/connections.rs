use crate::extensions::contracts::*;

#[test]
fn changing_or_removing_account_disables_connection_and_clears_reviewed_permissions() {
    let original = McpConnection {
        id: "connection".into(),
        connector_id: "service".into(),
        name: "Work".into(),
        revision: "one".into(),
        account_id: Some("old-account".into()),
        enabled: true,
        tools: vec![McpToolPermission {
            name: "publish".into(),
            schema_digest: "reviewed".into(),
            read_only: false,
            policy: McpToolPolicy::Allow,
        }],
    };
    for account in [Some("new-account".into()), None] {
        let mut changed = original.clone();
        changed.account_id = account;
        changed.apply_binding_change(&original);
        assert!(!changed.enabled);
        assert!(changed.tools.is_empty());
    }
    let mut unchanged = original.clone();
    unchanged.name = "Renamed".into();
    unchanged.apply_binding_change(&original);
    assert!(unchanged.enabled);
    assert_eq!(unchanged.tools.len(), 1);
}
