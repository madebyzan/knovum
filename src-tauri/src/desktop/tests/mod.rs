use serde_json::Value;
#[test]
fn renderer_capabilities_cannot_take_over_persistence_or_webview_sql() {
    let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("capabilities");
    for entry in std::fs::read_dir(directory).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().and_then(|p| p.to_str()) != Some("json") {
            continue;
        }
        let capability: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        for permission in capability["permissions"].as_array().unwrap() {
            let name = permission
                .as_str()
                .unwrap_or_else(|| permission["identifier"].as_str().unwrap());
            assert!(!name.starts_with("sql:"), "{}: {name}", path.display());
            if capability["identifier"] == "main-desktop" {
                assert_ne!(name, "fs:default");
            }
            if capability["identifier"] == "spaces" {
                assert!(
                    !["fs:allow-write-file", "fs:allow-mkdir", "fs:allow-remove"].contains(&name)
                );
            }
        }
    }
}
