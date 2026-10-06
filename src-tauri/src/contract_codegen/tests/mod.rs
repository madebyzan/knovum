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
fn committed_contracts_match_rust_sources() {
    use std::{
        collections::BTreeMap,
        path::{Path, PathBuf},
    };

    struct TemporaryRoot(PathBuf);
    impl Drop for TemporaryRoot {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    fn files(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
        fn visit(root: &Path, directory: &Path, output: &mut BTreeMap<PathBuf, Vec<u8>>) {
            for entry in std::fs::read_dir(directory).expect("generated contract directory") {
                let entry = entry.expect("generated contract entry");
                if entry.file_type().expect("contract entry type").is_dir() {
                    visit(root, &entry.path(), output);
                } else {
                    output.insert(
                        entry
                            .path()
                            .strip_prefix(root)
                            .expect("contract path")
                            .to_owned(),
                        std::fs::read(entry.path()).expect("generated contract"),
                    );
                }
            }
        }
        let mut output = BTreeMap::new();
        visit(root, root, &mut output);
        output
    }

    let repository = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("project root");
    let temporary = TemporaryRoot(
        std::env::temp_dir().join(format!("knovum-contract-check-{}", uuid::Uuid::new_v4())),
    );
    for directory in [
        "src/shared/contracts/generated",
        "src/features/spaces/contracts/generated",
        "python/schemas",
    ] {
        std::fs::create_dir_all(temporary.0.join(directory)).expect("temporary contract directory");
    }
    super::export(&temporary.0).expect("Rust contracts must export");
    for directory in [
        "src/shared/contracts/generated",
        "src/features/spaces/contracts/generated",
        "python/schemas",
    ] {
        let committed = files(&repository.join(directory));
        let generated = files(&temporary.0.join(directory));
        let differences = committed
            .keys()
            .chain(generated.keys())
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .filter(|path| committed.get(*path) != generated.get(*path))
            .collect::<Vec<_>>();
        assert!(
            differences.is_empty(),
            "Committed contracts differ from Rust sources in {directory}: {differences:?}; run pnpm contracts:generate"
        );
    }
    let agent_values = Path::new("python/src/knovum_agent_runtime/contracts/values.py");
    assert_eq!(
        std::fs::read(repository.join(agent_values))
            .expect("committed Agent Runtime protocol values"),
        std::fs::read(temporary.0.join(agent_values))
            .expect("generated Agent Runtime protocol values"),
        "Agent Runtime protocol values differ from Rust sources; run pnpm contracts:generate"
    );
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
        crate::ai::contracts::ExternalAgentKind::output_path(),
        Some(std::path::PathBuf::from("ExternalAgentKind.ts"))
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
    assert_identity::<crate::spaces::conversations::Conversation>();
}
