#[test]
fn provider_identity_round_trips() {
    use crate::ai::contracts::ProviderKind;
    let schema = schemars::schema_for!(ProviderKind);
    let values = schema.get("enum").unwrap().as_array().unwrap();
    assert!(!values.is_empty());
    for value in values {
        let name = value.as_str().unwrap();
        let provider: ProviderKind = serde_json::from_value(value.clone()).unwrap();
        assert_eq!(ProviderKind::parse(name), Some(provider));
        assert_eq!(provider.as_str(), name);
        assert_eq!(serde_json::to_value(provider).unwrap(), *value);
    }
    assert_eq!(ProviderKind::parse("unknown-provider"), None);
}

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
        crate::spaces::derivation::video_edit::RenderVideoEditRequest::output_path(),
        crate::spaces::derivation::video_edit::PrepareVideoEditProxyRequest::output_path(),
        crate::spaces::derivation::video_edit::VideoEditProgress::output_path(),
        crate::spaces::derivation::video_edit::fonts::InstalledFont::output_path(),
        crate::spaces::derivation::CompletedDerivationOutput::output_path(),
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

#[test]
fn exported_read_model_files_match_their_type_identity() {
    fn assert_identity<T: ts_rs::TS>() {
        let output = T::output_path().expect("exported summary path");
        assert_eq!(
            output.file_stem().and_then(|name| name.to_str()),
            Some(T::name(&ts_rs::Config::new()).as_str())
        );
    }
    assert_identity::<crate::spaces::runs::StepObservation>();
    assert_identity::<crate::spaces::assistant::AssistantConversation>();
}
