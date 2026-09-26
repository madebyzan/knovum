#[test]
#[ignore = "explicit repository contract generation"]
fn export_space_contracts() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("src-tauri must have a project root");
    super::export(root).expect("Space contracts must export");
}

#[test]
fn contracts_keep_their_capability_owner() {
    use ts_rs::TS;
    let spaces = std::path::Path::new("../../../features/spaces/contracts/generated");
    for path in [
        crate::spaces::structure::edit_contract::ExecuteSpaceEditRequest::output_path(),
        crate::spaces::actions::SpaceActionDefinition::output_path(),
        crate::spaces::agent_artifacts::contracts::AgentCreationRequest::output_path(),
    ] {
        assert!(path
            .expect("exported business contract")
            .starts_with(spaces));
    }
    assert_eq!(
        crate::ai::contracts::AgentKind::output_path(),
        Some(std::path::PathBuf::from("AgentKind.ts"))
    );
}
