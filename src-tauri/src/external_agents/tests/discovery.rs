use super::*;

#[cfg(target_os = "macos")]
#[test]
fn extracts_delimited_path() {
    let text = "banner\n__KNOVUM_PATH__:/opt/homebrew/bin:/usr/bin:__KNOVUM_PATH__\ntrailing";
    assert_eq!(
        extract_delimited(text, "__KNOVUM_PATH__:").as_deref(),
        Some("/opt/homebrew/bin:/usr/bin")
    );
}

#[test]
fn find_executable_resolves_existing_binary() {
    let path = std::env::var("PATH").unwrap_or_default();
    #[cfg(unix)]
    let binary = "sh";
    #[cfg(windows)]
    let binary = "cmd";
    assert!(find_executable(binary, &path).is_some());
}

#[test]
fn find_executable_returns_none_for_nonexistent() {
    assert!(find_executable("knovum_nonexistent_binary_xyz_123", "").is_none());
}
