use serde_json::json;

#[test]
fn discovery_tools_do_not_require_preselected_resource_ids() {
    for mode in [
        super::AssistantMode::Ask,
        super::AssistantMode::Plan,
        super::AssistantMode::Execute,
    ] {
        let catalog = super::project(mode).unwrap();
        for (id, parameters) in [
            ("list-space-contents", json!({"offset":0,"limit":40})),
            ("search-knowledge", json!({"query":"图片","limit":20})),
            ("read-node", json!({"nodeId":"known-node"})),
        ] {
            let definition = crate::spaces::actions::require(id).unwrap();
            assert!(definition.input_roles.is_empty());
            let tool = catalog
                .tools
                .iter()
                .find(|tool| catalog.by_wire_name[&tool.name].id == id)
                .unwrap();
            assert!(tool.input_schema["properties"]
                .get("inputNodeIds")
                .is_none());
            let validator = jsonschema::validator_for(&tool.input_schema).unwrap();
            assert!(validator.is_valid(&parameters), "{id}");
        }
        let create = catalog
            .by_wire_name
            .values()
            .any(|definition| definition.id == "create-note");
        assert_eq!(create, matches!(mode, super::AssistantMode::Execute));
    }
}
