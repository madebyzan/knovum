pub(crate) fn embed_manifest() {
    use sha2::{Digest, Sha256};
    use std::{fs, path::PathBuf};
    let root = PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap()).join("..");
    let target = match std::env::var("TARGET").unwrap().as_str() {
        "aarch64-apple-darwin" => "macos-arm64",
        "x86_64-apple-darwin" => "macos-x64",
        "x86_64-pc-windows-msvc" => "windows-x64",
        other => panic!("Unsupported inference target: {other}"),
    };
    let mut hash = Sha256::new();
    for file in [
        "inference/onnx-source.json",
        "inference/audio-source.json",
        "inference/sherpa-source.json",
        "inference/whisper-source.json",
        "inference/preparation.mjs",
        "inference/suppliers.mjs",
        "inference/onnx.mjs",
        "inference/audio.mjs",
        "inference/speech.mjs",
        "inference/whisper.mjs",
        "inference/source-archive.mjs",
        "selected-zip.mjs",
        "inference/libraries.mjs",
        "inference/voice_bridge.cmake",
        "inference/voice_bridge.cpp",
        "transfer.mjs",
        "files.mjs",
    ] {
        let path = root.join("scripts/runtime").join(file);
        println!("cargo:rerun-if-changed={}", path.display());
        hash.update(file.as_bytes());
        hash.update(b"\0");
        hash.update(fs::read(path).unwrap());
    }
    let configuration = root.join("src-tauri/tauri.conf.json");
    println!("cargo:rerun-if-changed={}", configuration.display());
    hash.update(fs::read(configuration).unwrap());
    let directory = root
        .join(".runtime-development/inference")
        .join(target)
        .join(hex::encode(hash.finalize()));
    println!(
        "cargo:rustc-env=KNOVUM_INFERENCE_RUNTIME_DIRECTORY={}",
        directory.display()
    );
    let path = if std::env::var("PROFILE").as_deref() == Ok("release") {
        root.join("src-tauri/resources/inference-runtime/inference-runtime-manifest.json")
    } else {
        directory.join("inference-runtime-manifest.json")
    };
    println!("cargo:rerun-if-changed={}", path.display());
    let manifest = fs::read(&path).expect(
        "Run node scripts/runtime/inference/preparation.mjs before building the desktop host",
    );
    let value: serde_json::Value =
        serde_json::from_slice(&manifest).expect("Prepared inference manifest must be valid JSON");
    assert_eq!(
        value["target"], target,
        "Prepared inference runtime target differs"
    );
    fs::write(
        PathBuf::from(std::env::var_os("OUT_DIR").unwrap()).join("inference-runtime-manifest.json"),
        manifest,
    )
    .unwrap();
}
