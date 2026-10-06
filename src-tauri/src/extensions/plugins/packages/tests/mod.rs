use crate::extensions::content::PackageFiles;
use serde_json::json;
fn package() -> PackageFiles {
    [("plugin.json".into(),serde_json::to_vec(&json!({"$schema":"https://agent-plugins.org/schemas/1.0.0/plugin.schema.json","name":"example","version":"preview-a"})).unwrap()),
    ("skills/example/SKILL.md".into(),b"---\nname: example\ndescription: Useful reference\n---\n\nRead selected resources.".to_vec())].into_iter().collect()
}
#[test]
fn malformed_component_does_not_discard_valid_skills() {
    let mut files = package();
    files.insert("mcp.json".into(), b"{invalid}".to_vec());
    let reviewed = super::inspect(&files, "revision".into()).unwrap();
    assert_eq!(reviewed.skills.len(), 1);
    assert!(reviewed.inspection.connectors.is_empty());
    assert!(reviewed
        .inspection
        .components
        .iter()
        .any(|item| item.name == "mcp.json" && !item.supported));
    assert_eq!(
        reviewed.inspection.manifest.version.as_deref(),
        Some("preview-a")
    );
}
#[test]
fn unknown_standard_extension_is_diagnosed_and_private_manifest_is_rejected() {
    let mut files = package();
    let mut manifest: serde_json::Value = serde_json::from_slice(&files["plugin.json"]).unwrap();
    manifest["extensions"] = json!({"vendor.unknown":{"setting":true}});
    files.insert("plugin.json".into(), serde_json::to_vec(&manifest).unwrap());
    assert!(super::inspect(&files, "revision".into())
        .unwrap()
        .inspection
        .components
        .iter()
        .any(|item| item.name == "vendor.unknown" && !item.supported));
    files.remove("plugin.json");
    files.insert(
        "knovum-plugin.json".into(),
        serde_json::to_vec(&manifest).unwrap(),
    );
    assert!(super::inspect(&files, "revision".into()).is_err());
}
#[test]
fn skill_directory_identity_and_duplicate_frontmatter_are_rejected() {
    for markdown in [
        "---\nname: wrong\ndescription: reference\n---\n",
        "---\nname: example\nname: example\ndescription: reference\n---\n",
    ] {
        let mut files = package();
        files.insert(
            "skills/example/SKILL.md".into(),
            markdown.as_bytes().to_vec(),
        );
        let reviewed = super::inspect(&files, "revision".into()).unwrap();
        assert!(reviewed.skills.is_empty());
        assert!(reviewed
            .inspection
            .components
            .iter()
            .any(|item| item.kind == "skill" && !item.supported));
    }
}

#[test]
fn plugin_service_metadata_is_bound_to_the_standard_mcp_component() {
    let mut files = package();
    let mut manifest: serde_json::Value = serde_json::from_slice(&files["plugin.json"]).unwrap();
    manifest["extensions"] = json!({"com.zand.knovum":{"connectors":{"github":{"name":"GitHub","description":"Read issues and repositories","authorization":{"type":"bearer"}}}}});
    files.insert("plugin.json".into(), serde_json::to_vec(&manifest).unwrap());
    files.insert("mcp.json".into(), serde_json::to_vec(&json!({"$schema":"https://agent-plugins.org/schemas/1.0.0/mcp.schema.json","mcpServers":{"github":{"type":"streamable-http","url":"https://example.com/mcp"}}})).unwrap());
    let reviewed = super::inspect(&files, "revision".into()).unwrap();
    let connector = &reviewed.inspection.connectors[0];
    assert_eq!(connector.component_name, "github");
    assert_eq!(connector.name, "GitHub");
    assert_eq!(connector.description, "Read issues and repositories");
    assert_eq!(
        connector.authorization,
        crate::extensions::contracts::ConnectorAuthorization::Bearer
    );
    assert_eq!(connector.configuration["url"], "https://example.com/mcp");
}

#[test]
fn stdio_declaration_preserves_valid_skills_before_machine_configuration() {
    let mut files = package();
    files.insert("mcp.json".into(), serde_json::to_vec(&json!({"$schema":"https://agent-plugins.org/schemas/1.0.0/mcp.schema.json","mcpServers":{"local":{"type":"stdio","command":"node","args":["server.js"]}}})).unwrap());
    let reviewed = super::inspect(&files, "one".into()).unwrap();
    assert_eq!(reviewed.skills.len(), 1);
    assert_eq!(reviewed.inspection.connectors.len(), 1);
    assert_eq!(
        reviewed.inspection.connectors[0].configuration["command"],
        "node"
    );
    assert!(reviewed
        .inspection
        .components
        .iter()
        .any(|component| component.kind == "mcp" && component.supported));
}

#[test]
fn incompatible_stdio_auth_is_diagnosed_without_discarding_independent_skills() {
    let mut files = package();
    let mut manifest: serde_json::Value = serde_json::from_slice(&files["plugin.json"]).unwrap();
    manifest["extensions"] = json!({"com.zand.knovum":{"connectors":{"local":{"name":"Local","description":"Reference","authorization":{"type":"bearer"}}}}});
    files.insert("plugin.json".into(), serde_json::to_vec(&manifest).unwrap());
    files.insert("mcp.json".into(), serde_json::to_vec(&json!({"$schema":"https://agent-plugins.org/schemas/1.0.0/mcp.schema.json","mcpServers":{"local":{"type":"stdio","command":"node"}}})).unwrap());
    let reviewed = super::inspect(&files, "one".into()).unwrap();
    assert_eq!(reviewed.skills.len(), 1);
    assert!(reviewed.inspection.connectors.is_empty());
    assert!(reviewed
        .inspection
        .components
        .iter()
        .any(|component| component.name == "local" && !component.supported));
}

#[test]
fn invalid_http_secret_fields_are_diagnosed_before_installation() {
    for names in [
        vec!["bad name"],
        vec!["mcp-session-id"],
        vec!["X-Key", "x-key"],
        vec![],
    ] {
        let mut files = package();
        let mut manifest: serde_json::Value =
            serde_json::from_slice(&files["plugin.json"]).unwrap();
        manifest["extensions"] = json!({"com.zand.knovum":{"connectors":{"remote":{"name":"Remote","description":"Reference","authorization":{"type":"secret_fields","names":names}}}}});
        files.insert("plugin.json".into(), serde_json::to_vec(&manifest).unwrap());
        files.insert("mcp.json".into(), serde_json::to_vec(&json!({"$schema":"https://agent-plugins.org/schemas/1.0.0/mcp.schema.json","mcpServers":{"remote":{"type":"streamable-http","url":"https://example.com/mcp"}}})).unwrap());
        let reviewed = super::inspect(&files, "one".into()).unwrap();
        assert_eq!(reviewed.skills.len(), 1);
        assert!(reviewed.inspection.connectors.is_empty());
        assert!(reviewed.inspection.components.iter().any(|component| {
            component.kind == "mcp" && component.name == "remote" && !component.supported
        }));
    }
}
