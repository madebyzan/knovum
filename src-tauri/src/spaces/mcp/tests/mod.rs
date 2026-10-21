#[test]
fn every_listed_tool_schema_is_mcp_conformant() {
    // One boolean property schema makes reference-SDK clients reject the whole listing, which
    // leaves a desktop Agent with no Knovum tool at all.
    for tool in super::SpaceMcpServer::tool_router().list_all() {
        let mut schema = (*tool.input_schema).clone();
        crate::extensions::mcp::tool_schema::conform(&mut schema);
        assert_eq!(
            schema, *tool.input_schema,
            "{} declares a boolean property schema",
            tool.name
        );
    }
}

#[test]
fn change_set_tools_publish_the_operations_they_accept() {
    let tools = super::SpaceMcpServer::tool_router().list_all();
    let commit = tools
        .iter()
        .find(|tool| tool.name == "commit_knowledge_changes")
        .expect("registered commit tool");
    assert_eq!(
        commit.input_schema["properties"]["operations"]["type"],
        "array"
    );
    let invoke = tools
        .iter()
        .find(|tool| tool.name == "invoke_space_action")
        .expect("registered invoke tool");
    for field in ["parameters", "input_snapshot", "expected_revisions"] {
        assert_eq!(invoke.input_schema["properties"][field]["type"], "object");
    }
}

#[test]
fn canvas_read_tools_are_discoverable_with_space_scoped_arguments() {
    let tools = super::SpaceMcpServer::tool_router().list_all();
    for name in [
        "list_space_contents",
        "search_knowledge",
        "get_node",
        "get_node_asset",
    ] {
        let tool = tools
            .iter()
            .find(|tool| tool.name == name)
            .expect("registered read tool");
        assert!(tool.input_schema["properties"].get("space_id").is_some());
        assert!(tool.annotations.as_ref().unwrap().read_only_hint.unwrap());
        if name == "get_node_asset" {
            assert!(tool.input_schema["required"]
                .as_array()
                .unwrap()
                .iter()
                .any(|field| field == "asset_revision"));
        }
    }
}
