#[path = "../../build_support/cmake_install.rs"]
mod cmake_install;
fn main() {
    println!("cargo:rerun-if-changed=cmake");
    println!("cargo:rerun-if-changed=../../build_support/cmake_install.rs");
    println!("cargo:rerun-if-changed=../../build_support/windows-system-libraries.json");
    let triple = std::env::var("TARGET").unwrap();
    let target = match triple.as_str() {
        "aarch64-apple-darwin" => "macos-arm64",
        "x86_64-apple-darwin" => "macos-x64",
        "x86_64-pc-windows-msvc" => "windows-x64",
        other => panic!("Unsupported media target: {other}"),
    };
    let root = std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap()).join("runtime");
    cmake_install::install(&triple, target, &root);
    println!(
        "cargo:rustc-env=KNOVUM_MEDIA_LIBRARIES_DIRECTORY={}",
        root.display()
    );
    println!("cargo::metadata=directory={}", root.display());
}
