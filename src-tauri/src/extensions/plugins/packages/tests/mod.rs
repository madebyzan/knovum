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
    assert!(reviewed.mcp_servers.is_empty());
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
