pub(super) fn embed_descriptors() {
    for profile in [
        "resources",
        "light-inference",
        "torch-inference",
        "voice-generation",
        "agent",
    ] {
        let key = format!(
            "KNOVUM_PYTHON_{}_DESCRIPTOR_PATH",
            profile.to_ascii_uppercase().replace('-', "_")
        );
        println!("cargo:rerun-if-env-changed={key}");
        let bytes = match std::env::var_os(&key) {
            Some(path) => {
                println!(
                    "cargo:rerun-if-changed={}",
                    std::path::Path::new(&path).display()
                );
                std::fs::read(path).expect("read trusted Python runtime descriptor")
            }
            None if std::env::var("PROFILE").as_deref() == Ok("release")
                && std::env::var_os("CARGO_FEATURE_E2E").is_none() =>
            {
                panic!("release requires {key}")
            }
            None => b"null".to_vec(),
        };
        let destination = std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap()).join(
            format!("python-{}-descriptor.json", profile.to_ascii_lowercase()),
        );
        std::fs::write(destination, bytes).expect("embed trusted Python runtime descriptor");
    }
}
