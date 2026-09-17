use super::RestorePackageRequest;

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
        "creationWorkflowRuns": [],
        "provenance": [],
        "knowledgeEvents": [],
        "workLinks": [],
        "chunks": [],
        "threads": [],
        "messages": []
    });

    let decoded = serde_json::from_value::<RestorePackageRequest>(value);

    assert!(decoded.is_err());
}
