pub(crate) fn embed_manifest() {
    use std::{fs, path::PathBuf};
    let directory = PathBuf::from(
        std::env::var_os("DEP_KNOVUM_INFERENCE_LIBRARIES_DIRECTORY")
            .expect("Cargo must build the compiled inference dependency"),
    );
    println!(
        "cargo:rustc-env=KNOVUM_INFERENCE_RUNTIME_DIRECTORY={}",
        directory.display()
    );
    let path = if std::env::var_os("CARGO_CFG_DEBUG_ASSERTIONS").is_some() {
        directory.join("inference-runtime-manifest.json")
    } else {
        PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap())
            .join("resources/inference-runtime/inference-runtime-manifest.json")
    };
    println!("cargo:rerun-if-changed={}", path.display());
    let manifest = fs::read(&path)
        .expect("Distribution builds require compiled resources assembled by pnpm package");
    let value: serde_json::Value =
        serde_json::from_slice(&manifest).expect("Compiled manifest must be JSON");
    assert_eq!(value["schemaVersion"], 1);
    let target = match std::env::var("TARGET").unwrap().as_str() {
        "aarch64-apple-darwin" => "macos-arm64",
        "x86_64-apple-darwin" => "macos-x64",
        "x86_64-pc-windows-msvc" => "windows-x64",
        other => panic!("Unsupported compiled target: {other}"),
    };
    assert_eq!(
        value["target"], target,
        "Compiled resources must match the Cargo target"
    );
    fs::write(
        PathBuf::from(std::env::var_os("OUT_DIR").unwrap()).join("inference-runtime-manifest.json"),
        manifest,
    )
    .unwrap();
}
