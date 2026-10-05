#[test]
fn asset_identity_does_not_substitute_knowledge_or_original_file_revisions() {
    let node = serde_json::json!({"assetRevision":"rendered1:current","knowledgeRevision":"node1:content","sourceRevision":"sha256:file"});
    assert!(super::require_revision(&node, "rendered1:current").is_ok());
    for revision in ["rendered1:old", "node1:content", "sha256:file"] {
        assert!(super::require_revision(&node, revision).is_err());
    }
    assert!(super::require_revision(
        &serde_json::json!({"sourceRevision":"sha256:file"}),
        "sha256:file"
    )
    .is_err());
}
