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
