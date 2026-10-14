use super::super::links::validate;

#[test]
fn allows_web_and_mail_schemes() {
    assert!(validate("https://example.com").is_ok());
    assert!(validate("http://example.com/path?query=1#frag").is_ok());
    assert!(validate("mailto:author@example.com").is_ok());
    assert!(validate("HTTPS://UPPERCASE.COM").is_ok());
}

#[test]
fn allows_fragment_anchors_and_relative_paths() {
    assert!(validate("#heading-1").is_ok());
    assert!(validate("#_Toc123456").is_ok());
    assert!(validate("report.pdf").is_ok());
    assert!(validate("./chapter1.docx").is_ok());
    assert!(validate("../appendix.pdf").is_ok());
    assert!(validate("docs/spec.html#section").is_ok());
    assert!(validate("path/with:colon").is_ok());
}

#[test]
fn rejects_unsafe_schemes_and_absolute_local_paths() {
    assert!(validate("javascript:alert(1)").is_err());
    assert!(validate("file:///etc/passwd").is_err());
    assert!(validate("file:///C:/Users").is_err());
    assert!(validate("/private/document").is_err());
    assert!(validate("/etc/hosts").is_err());
    assert!(validate("\\\\server\\share").is_err());
    assert!(validate("C:\\Windows\\System32").is_err());
    assert!(validate("data:text/html,hello").is_err());
    assert!(validate("").is_err());
    assert!(validate("https://example.com\0evil").is_err());
}

#[test]
fn rejects_windows_drive_paths_with_the_correct_message() {
    // A single ASCII letter is syntactically a valid URI scheme, so without
    // an explicit drive-letter check `extract_scheme` would parse "C" out of
    // "C:\Windows\..." as an unrecognized *scheme* and reject it with the
    // wrong message. Both backslash and forward-slash drive paths must be
    // reported as absolute local filesystem paths.
    let backslash = validate("C:\\Windows\\System32").unwrap_err();
    let forward_slash = validate("C:/Users/foo/secret.docx").unwrap_err();
    assert!(backslash.contains("absolute local filesystem paths"), "{backslash}");
    assert!(forward_slash.contains("absolute local filesystem paths"), "{forward_slash}");
}
