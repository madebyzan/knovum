pub(crate) fn embed_manifest() {
    use sha2::{Digest, Sha256};
    use std::{fs, path::PathBuf};
    let root = PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap()).join("..");
    let target = match std::env::var("TARGET").unwrap().as_str() {
        "aarch64-apple-darwin" => "macos-arm64",
        "x86_64-apple-darwin" => "macos-x64",
        "x86_64-pc-windows-msvc" => "windows-x64",
        other => panic!("Unsupported native target: {other}"),
    };
    let mut hash = Sha256::new();
    for file in [
        "native-source.json",
        "native-audio-source.json",
        "native-sherpa-source.json",
        "native-whisper-source.json",
        "native.mjs",
        "native-ort.mjs",
        "native-audio.mjs",
        "native-speech.mjs",
        "native-whisper.mjs",
        "native-source-zip.mjs",
        "selected-zip.mjs",
        "native-closure.mjs",
        "voice_bridge.cmake",
        "voice_bridge.cpp",
    ] {
        let path = root.join("scripts/runtime").join(file);
        println!("cargo:rerun-if-changed={}", path.display());
        hash.update(fs::read(path).unwrap());
    }
    let directory = root
        .join(".runtime-development/native")
        .join(target)
        .join(hex::encode(hash.finalize()));
    println!(
        "cargo:rustc-env=KNOVUM_NATIVE_RUNTIME_DIRECTORY={}",
        directory.display()
    );
    let path = if std::env::var("PROFILE").as_deref() == Ok("release") {
        root.join("src-tauri/resources/native-runtime/native-runtime-manifest.json")
    } else {
        directory.join("native-runtime-manifest.json")
    };
    println!("cargo:rerun-if-changed={}", path.display());
    let manifest = fs::read(&path)
        .expect("Run node scripts/runtime/native.mjs before building the desktop host");
    let value: serde_json::Value =
        serde_json::from_slice(&manifest).expect("Prepared native manifest must be valid JSON");
    assert_eq!(
        value["target"], target,
        "Prepared native runtime target differs"
    );
    fs::write(
        PathBuf::from(std::env::var_os("OUT_DIR").unwrap()).join("native-runtime-manifest.json"),
        manifest,
    )
    .unwrap();
}
