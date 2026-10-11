/// The application bundles one frozen Agent; Cargo never invokes its environment manager.
pub(crate) fn embed_manifest() {
    let root = std::path::PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let path = root.join("resources/agent-runtime/knovum-agent/agent-manifest.json");
    let bytes = if std::env::var_os("CARGO_CFG_DEBUG_ASSERTIONS").is_some() {
        b"null".to_vec()
    } else {
        println!("cargo:rerun-if-changed={}", path.display());
        std::fs::read(&path)
            .expect("Build the frozen Agent through pnpm package before a distribution build")
    };
    std::fs::write(
        std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap()).join("agent-manifest.json"),
        bytes,
    )
    .unwrap();
}
