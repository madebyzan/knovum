pub(crate) fn embed_descriptors() {
    let root =
        std::path::PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap()).join("../python");
    let source_path = root.join("runtime-source.json");
    println!("cargo:rerun-if-changed={}", source_path.display());
    let source: serde_json::Value = serde_json::from_slice(&std::fs::read(&source_path).unwrap())
        .expect("read Conda runtime supplier configuration");
    let target = match std::env::var("TARGET").unwrap().as_str() {
        "aarch64-apple-darwin" => "macos-arm64",
        "x86_64-apple-darwin" => "macos-x64",
        "x86_64-pc-windows-msvc" => "windows-x64",
        other => panic!("Unsupported desktop target: {other}"),
    };
    for profile in [
        "resources",
        "light-inference",
        "torch-inference",
        "voice-generation",
        "agent",
    ] {
        embed_producer_sources(&root, &source, profile);
        embed_recipe(&root, target, profile);
        let group = source["condaSupplier"]["profiles"][profile]
            .as_str()
            .expect("Conda runtime profile group");
        let mut dependency_bytes = Vec::new();
        for path in [
            root.join("runtimes").join(profile).join("uv.lock"),
            root.join("conda-locks")
                .join(group)
                .join(format!("{target}.json")),
        ] {
            println!("cargo:rerun-if-changed={}", path.display());
            dependency_bytes
                .extend(std::fs::read(path).expect("read Conda runtime dependency lock"));
        }
        let destination = std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap())
            .join(format!("python-{profile}-dependencies.lock"));
        std::fs::write(destination, dependency_bytes).expect("embed Conda runtime dependency lock");
        let descriptor = root
            .join("../src-tauri/resources")
            .join(format!("python-{profile}-descriptor.json"));
        println!("cargo:rerun-if-changed={}", descriptor.display());
        let bytes = match std::fs::read(&descriptor) {
            Ok(bytes) => bytes,
            Err(error)
                if error.kind() == std::io::ErrorKind::NotFound
                    && (std::env::var("PROFILE").as_deref() != Ok("release")
                        || std::env::var_os("CARGO_FEATURE_E2E").is_some()) =>
            {
                b"null".to_vec()
            }
            Err(error) => panic!(
                "Required Python runtime descriptor {}: {error}",
                descriptor.display()
            ),
        };
        let destination = std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap()).join(
            format!("python-{}-descriptor.json", profile.to_ascii_lowercase()),
        );
        std::fs::write(destination, bytes).expect("embed trusted Python runtime descriptor");
    }
}

fn embed_recipe(root: &std::path::Path, target: &str, profile: &str) {
    let repository = root
        .parent()
        .expect("Python project belongs to the repository");
    if profile == "light-inference" {
        println!(
            "cargo:rerun-if-changed={}",
            root.join("compiled-source-evidence.json").display()
        );
    }
    if profile == "voice-generation" && target.starts_with("macos-") {
        println!(
            "cargo:rerun-if-changed={}",
            repository.join("src-tauri/tauri.conf.json").display()
        );
    }
    println!(
        "cargo:rerun-if-changed={}",
        repository.join("scripts/runtime/profiles.mjs").display()
    );
    let output = std::process::Command::new("node")
        .arg(repository.join("scripts/runtime/profiles.mjs"))
        .arg("--repository")
        .arg(repository)
        .args(["--target", target, "--profile", profile])
        .output()
        .expect("Node is required by the desktop runtime producer");
    assert!(
        output.status.success(),
        "Runtime recipe failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let destination = std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap())
        .join(format!("python-{profile}-recipe.json"));
    std::fs::write(destination, output.stdout).expect("embed selected runtime recipe");
}

fn embed_producer_sources(root: &std::path::Path, source: &serde_json::Value, profile: &str) {
    let sources = &source["producerSources"];
    let mut names = sources["common"]
        .as_array()
        .unwrap()
        .iter()
        .chain(sources["python"].as_array().unwrap())
        .chain(
            sources["profiles"][profile]
                .as_array()
                .into_iter()
                .flatten(),
        )
        .map(|name| name.as_str().unwrap())
        .collect::<Vec<_>>();
    names.sort();
    let mut bytes = Vec::new();
    for name in names {
        let path = root.join("../scripts/runtime").join(name);
        println!("cargo:rerun-if-changed={}", path.display());
        bytes.extend(name.as_bytes());
        bytes.push(0);
        bytes.extend(std::fs::read(path).expect("read runtime producer source"));
        bytes.push(0);
    }
    let output = std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
    std::fs::write(
        output.join(format!("python-{profile}-producer.source")),
        bytes,
    )
    .expect("embed runtime producer source identity");
}
