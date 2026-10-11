#[path = "../../build_support/cmake_install.rs"]
mod cmake_install;
#[path = "build_support/layout.rs"]
mod layout;
#[path = "build_support/relocation/mod.rs"]
mod relocation;
use std::{fs, path::PathBuf, process::Command};
fn main() {
    println!("cargo:rerun-if-changed=cmake");
    println!("cargo:rerun-if-changed=../../build_support/windows-system-libraries.json");
    println!("cargo:rerun-if-changed=build_support");
    println!("cargo:rerun-if-changed=../../build_support/cmake_install.rs");
    let target = std::env::var("TARGET").unwrap();
    let platform = match target.as_str() {
        "aarch64-apple-darwin" => "macos-arm64",
        "x86_64-apple-darwin" => "macos-x64",
        "x86_64-pc-windows-msvc" => "windows-x64",
        _ => panic!("Unsupported inference target: {target}"),
    };
    let root = PathBuf::from(std::env::var_os("OUT_DIR").unwrap()).join("runtime");
    cmake_install::install(&target, platform, &root);
    relocation::materialize(&root, &root)
        .expect("Compiled install aliases must remain inside their owner");
    let manifest_path = root.join("inference-runtime-manifest.json");
    if manifest_path.exists() {
        fs::remove_file(&manifest_path).unwrap();
    }
    let files =
        layout::inventory(&root).expect("Compiled install must contain bounded regular files");
    layout::require_libraries(&files, target.contains("windows"))
        .expect("Compiled supplier inventory must be complete");
    if target.contains("apple") {
        relocation::relocate(&root, &files)
            .expect("Compiled libraries must resolve only packaged/system dependencies");
        for file in files
            .iter()
            .filter(|p| p.ends_with(".dylib") || p.ends_with(".so"))
        {
            assert!(
                Command::new("codesign")
                    .args(["--force", "--sign", "-"])
                    .arg(root.join(file))
                    .status()
                    .unwrap()
                    .success(),
                "Compiled ad-hoc signing failed"
            );
        }
    }
    assert!(
        Command::new("cmake")
            .arg(format!("-DROOT={}", root.display()))
            .arg(format!("-DTARGET_PLATFORM={platform}"))
            .args(["-P", "cmake/manifest.cmake"])
            .status()
            .unwrap()
            .success(),
        "Inference runtime manifest generation failed"
    );
    println!(
        "cargo:rustc-env=KNOVUM_INFERENCE_LIBRARIES_DIRECTORY={}",
        root.display()
    );
    println!("cargo::metadata=directory={}", root.display());
    println!("cargo::metadata=manifest={}", manifest_path.display());
    fs::write(
        PathBuf::from(std::env::var_os("OUT_DIR").unwrap()).join("directory.txt"),
        root.to_str().unwrap(),
    )
    .unwrap();
}
