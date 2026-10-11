use super::{checked_worker_root, RuntimeProfile};

#[cfg(debug_assertions)]
#[test]
fn development_location_matches_the_agent_runtime_producer() {
    let repository = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap();
    let profile = RuntimeProfile::Agent;
    let output = std::process::Command::new("node")
            .args(["--input-type=module", "-e", "import {developmentDirectory,hostRuntimeTarget} from './scripts/runtime/profiles.mjs'; console.log(developmentDirectory(process.cwd(),hostRuntimeTarget(),process.argv[1]));", profile.name()])
            .current_dir(repository)
            .output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        profile.development_manifest(),
        std::path::PathBuf::from(String::from_utf8(output.stdout).unwrap().trim())
            .join("runtime/runtime-manifest.json")
    );
}

#[test]
fn worker_directory_requires_the_selected_profile_entry() {
    let root = std::env::temp_dir().join(format!("knovum-worker-{}", uuid::Uuid::new_v4()));
    let entry = root.join(RuntimeProfile::Agent.entry());
    std::fs::create_dir_all(entry.parent().unwrap()).unwrap();
    std::fs::write(&entry, "").unwrap();

    assert_eq!(
        checked_worker_root(root.clone(), RuntimeProfile::Agent).unwrap(),
        root
    );
    assert!(checked_worker_root(entry, RuntimeProfile::Agent).is_err());
    std::fs::remove_file(root.join(RuntimeProfile::Agent.entry())).unwrap();
    assert!(checked_worker_root(root.clone(), RuntimeProfile::Agent).is_err());
    std::fs::remove_dir_all(root).unwrap();
}
