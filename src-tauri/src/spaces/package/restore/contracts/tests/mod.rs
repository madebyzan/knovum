use super::RestorePackageRequest;

#[test]
fn package_conversations_reject_removed_modes_instead_of_reinterpreting_them() {
    let mut value = serde_json::json!({"id":"conversation","title":"Notes","activeBranchId":"branch",
        "archivedAt":null,"createdAt":1,"updatedAt":1});
    assert!(serde_json::from_value::<super::PackageConversation>(value.clone()).is_ok());
    for mode in ["ask", "plan", "execute"] {
        value["mode"] = serde_json::json!(mode);
        let error =
            serde_json::from_value::<super::PackageConversation>(value.clone()).unwrap_err();
        assert!(error.to_string().contains("unknown field `mode`"));
    }
}

#[test]
fn restore_package_rejects_removed_work_links_field() {
    let value = serde_json::json!({
        "space": {
            "id": "space",
            "kind": "space",
            "title": "Space",
            "lastOpenedAt": 1,
            "createdAt": 1,
            "updatedAt": 1
        },
        "objectTypes": [],
        "nodes": [],
        "scenes": [],
        "relations": [],
        "derivations": [],
        "runs": [],
        "runSteps": [],
        "runAttempts": [],
        "runEvents": [],
        "externalAgentConversations": [],
        "externalAgentTurns": [],
        "conversations": [],
        "conversationMessages": [],
        "provenance": [],
        "knowledgeEvents": [],
        "workLinks": [],
        "chunks": [],
    });

    let decoded = serde_json::from_value::<RestorePackageRequest>(value);

    assert!(decoded.is_err());
}
