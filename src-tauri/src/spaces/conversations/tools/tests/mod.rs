use serde_json::json;

#[test]
fn discovery_tools_do_not_require_preselected_resource_ids() {
    let catalog = super::project().unwrap();
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
                .find(|tool| matches!(&catalog.binding_by_wire_name[&tool.name].source, crate::extensions::contracts::AgentToolBinding::SpaceAction { action_id, .. } if action_id == id))
                .unwrap();
        assert!(tool.input_schema["properties"]
            .get("inputNodeIds")
            .is_none());
        let validator = jsonschema::validator_for(&tool.input_schema).unwrap();
        assert!(validator.is_valid(&parameters), "{id}");
    }
}

#[test]
fn standard_agent_tools_preserve_host_effect_classification() {
    let catalog = super::project().unwrap();
    let create = catalog
            .binding_by_wire_name
            .values()
            .any(|definition| matches!(&definition.source, crate::extensions::contracts::AgentToolBinding::SpaceAction { action_id, .. } if action_id == "create-note"));
    assert!(create);
    for binding in catalog.binding_by_wire_name.values() {
        if let crate::extensions::contracts::AgentToolBinding::SpaceAction { action_id, .. } =
            &binding.source
        {
            let definition = crate::spaces::actions::require(action_id).unwrap();
            assert_eq!(
                binding.read_only,
                definition.impact == crate::spaces::actions::SpaceActionImpact::ReadOnly
            );
        }
    }
}

#[test]
fn a_generation_tool_runs_on_its_prompt_alone() {
    let catalog = super::project().unwrap();
    let tool = catalog
        .tools
        .iter()
        .find(|tool| matches!(&catalog.binding_by_wire_name[&tool.name].source, crate::extensions::contracts::AgentToolBinding::SpaceAction { action_id, .. } if action_id == "draft-document"))
        .unwrap();
    assert_eq!(
        tool.input_schema["properties"]["inputNodeIds"]["minItems"],
        0
    );
    let validator = jsonschema::validator_for(&tool.input_schema).unwrap();
    assert!(validator.is_valid(&json!({"title":"Plan","purpose":"A launch plan"})));
}
