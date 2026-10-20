#[path = "../../../build_support/cmake_install.rs"]
pub mod cmake_install;
use std::path::Path;
const OUT_DIR: &str = r"D:\a\knovum\knovum\src-tauri\target\debug\build\knovum-inference-libraries-cca5c127199394d3\out";
#[test]
fn windows_supplier_builds_use_one_short_tree_per_out_directory() {
    let cache = Path::new(r"C:\Users\runneradmin\AppData\Local");
    let out = Path::new(OUT_DIR);
    let tree = cmake_install::windows_build_tree(cache, out);
    assert!(tree.starts_with(cache));
    assert!(tree.as_os_str().len() < out.as_os_str().len());
    assert_eq!(tree, cmake_install::windows_build_tree(cache, out));
    assert_ne!(
        tree,
        cmake_install::windows_build_tree(cache, &out.with_file_name("out2"))
    );
}
#[test]
#[should_panic(expected = "build tree within")]
fn an_overlong_cache_fails_before_the_suppliers_exceed_the_path_limit() {
    cmake_install::windows_build_tree(
        Path::new(r"C:\Users\a-very-long-build-cache-directory-name\AppData\Local"),
        Path::new(OUT_DIR),
    );
}
