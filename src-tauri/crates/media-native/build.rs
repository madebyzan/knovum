#[path = "../../build_support/native_cmake.rs"]
mod native_cmake;
fn main() {
    println!("cargo:rerun-if-changed=native");
    println!("cargo:rerun-if-changed=../../build_support/native_cmake.rs");
    println!("cargo:rerun-if-changed=../../build_support/windows-system-libraries.json");
    let triple = std::env::var("TARGET").unwrap();
    let target = match triple.as_str() {
        "aarch64-apple-darwin" => "macos-arm64",
        "x86_64-apple-darwin" => "macos-x64",
        "x86_64-pc-windows-msvc" => "windows-x64",
        other => panic!("Unsupported media target: {other}"),
    };
    let root = std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap()).join("runtime");
    native_cmake::install(&triple, target, &root);
    println!(
        "cargo:rustc-env=KNOVUM_MEDIA_NATIVE_DIRECTORY={}",
        root.display()
    );
    println!("cargo::metadata=directory={}", root.display());
}
