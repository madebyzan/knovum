use super::super::paths;
#[test]
fn disk_inventory_paths_cannot_escape_the_resource_root() {
    let root = std::path::Path::new("/bundle/Agent");
    assert!(paths::join(root, "_internal/library.so").is_ok());
    for input in [
        "../secret",
        "/absolute",
        "C:\\outside",
        "foo\\..\\secret",
        "",
        "./file",
        "a//file",
    ] {
        assert!(paths::join(root, input).is_err(), "{input}");
    }
}
