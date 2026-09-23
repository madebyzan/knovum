#[test]
#[ignore = "explicit repository contract generation"]
fn export_space_contracts() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("src-tauri must have a project root");
    super::export(root).expect("Space contracts must export");
}
