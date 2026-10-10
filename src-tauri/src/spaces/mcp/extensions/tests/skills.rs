use crate::extensions::contracts::{AgentExtensionSnapshot, SkillRevision};
use serde_json::json;

#[test]
fn discovery_exposes_only_each_selected_skills_admitted_tool_aliases() {
    let mut snapshot = AgentExtensionSnapshot {
        id: "snapshot".into(),
        digest: "reviewed".into(),
        plugins: vec![],
        skills: vec![SkillRevision {
            id: "skill".into(),
            plugin_id: Some("package".into()),
            name: "evidence".into(),
            description: "Read grounded evidence".into(),
            digest: "skill-revision".into(),
            enabled: true,
            requirements: vec!["sources".into()],
            included_tools: vec!["sources/search".into(), "sources/resources/read".into()],
            unsupported_requirements: vec![],
        }],
        files: vec![],
        skill_tool_bindings: [
            ("sources/search".into(), vec!["mcp_search_reviewed".into()]),
            (
                "sources/resources/read".into(),
                vec!["mcp_resources_read_reviewed".into()],
            ),
            (
                "another/publish".into(),
                vec!["mcp_publish_unrelated".into()],
            ),
        ]
        .into_iter()
        .collect(),
        connectors: vec![],
        connections: vec![],
        accounts: vec![],
        tools: vec![],
        catalogs: Default::default(),
    };
    let selected = serde_json::to_value(super::selected_skills(&snapshot).unwrap()).unwrap();
    assert_eq!(
        selected,
        json!([{
            "name":"evidence", "description":"Read grounded evidence",
            "path":"/skills/evidence/SKILL.md", "resources":[],
            "toolBindings":{
                "sources/search":["mcp_search_reviewed"],
                "sources/resources/read":["mcp_resources_read_reviewed"]
            }
        }])
    );
    snapshot.skill_tool_bindings.remove("sources/search");
    assert!(super::selected_skills(&snapshot).is_err());
}

#[test]
fn progressive_discovery_exposes_resource_paths_without_loading_content() {
    let mut snapshot: AgentExtensionSnapshot = serde_json::from_value(json!({
        "id":"snapshot", "digest":"reviewed", "plugins":[],
        "skills":[{"id":"skill", "pluginId":null, "name":"study", "description":"Grounded study", "digest":"one", "enabled":true, "requirements":[], "includedTools":[], "unsupportedRequirements":[]}],
        "files":[
            {"path":"/skills/study/SKILL.md", "encoding":"utf8", "content":"private instructions", "digest":"one"},
            {"path":"/skills/study/references/practice.md", "encoding":"utf8", "content":"reference body only on request", "digest":"two"},
            {"path":"/skills/study-other/templates/guide.md", "encoding":"utf8", "content":"another selected skill", "digest":"three"}
        ],
        "skillToolBindings":{}, "connectors":[], "connections":[], "accounts":[], "tools":[], "catalogs":{}
    })).unwrap();
    let discovery = serde_json::to_value(super::selected_skills(&snapshot).unwrap()).unwrap();
    assert_eq!(
        discovery[0]["resources"],
        json!(["/skills/study/references/practice.md"])
    );
    let encoded = discovery.to_string();
    assert!(!encoded.contains("private instructions"));
    assert!(!encoded.contains("reference body only on request"));
    snapshot.files.clear();
    assert!(super::selected_skills(&snapshot).unwrap()[0]
        .resources
        .is_empty());
}
