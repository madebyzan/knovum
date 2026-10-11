/// A test thread never dispatches to a main thread, so the panel is uninstalled here and the
/// contract under test is that AppKit being unreachable reports a message instead of panicking.
#[cfg(target_os = "macos")]
#[test]
fn preview_without_an_installed_panel_fails_visibly() {
    let error = super::macos::preview(std::path::Path::new("/Applications")).unwrap_err();
    assert!(error.contains("Quick Look"), "{error}");
}
