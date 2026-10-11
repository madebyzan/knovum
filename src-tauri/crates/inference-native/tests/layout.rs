#[path = "../build_support/layout.rs"]
mod layout;
#[path = "../build_support/relocation.rs"]
pub mod relocation;
use std::{fs, path::PathBuf};
fn fixture(name: &str) -> PathBuf {
    let root =
        std::env::temp_dir().join(format!("knovum-native-test-{}-{name}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("compiled-libraries/whisper")).unwrap();
    for name in [
        "libonnxruntime.1.22.1.dylib",
        "compiled-libraries/audiocpp.dylib",
        "compiled-libraries/knovum_voice.dylib",
        "compiled-libraries/sherpa-onnx-c-api.dylib",
        "compiled-libraries/whisper/libwhisper.dylib",
    ] {
        fs::write(root.join(name), b"native bytes").unwrap();
    }
    root
}
#[test]
fn incomplete_and_linked_layouts_fail_before_sealing() {
    let root = fixture("inventory");
    let files = layout::inventory(&root).unwrap();
    layout::require_libraries(&files, false).unwrap();
    fs::remove_file(root.join("compiled-libraries/knovum_voice.dylib")).unwrap();
    assert!(layout::require_libraries(&layout::inventory(&root).unwrap(), false).is_err());
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink("/etc/passwd", root.join("escaped.dylib")).unwrap();
        assert!(layout::inventory(&root).is_err());
    }
    fs::remove_dir_all(root).unwrap();
}

#[cfg(unix)]
#[test]
fn materializes_only_aliases_within_the_installed_supplier_tree() {
    let root = fixture("aliases");
    let alias = root.join("alias.dylib");
    std::os::unix::fs::symlink("libonnxruntime.1.22.1.dylib", &alias).unwrap();
    relocation::materialize(&root, &root).unwrap();
    assert!(!fs::symlink_metadata(&alias).unwrap().is_symlink());
    assert_eq!(fs::read(&alias).unwrap(), b"native bytes");
    std::os::unix::fs::symlink("/etc/passwd", root.join("escape.dylib")).unwrap();
    assert!(relocation::materialize(&root, &root).is_err());
    fs::remove_dir_all(root).unwrap();
}
