fn extract_scheme(url: &str) -> Option<&str> {
    let colon_pos = url.find(':')?;
    if let Some(sep) = url.find(['/', '?', '#']) {
        if sep < colon_pos {
            return None;
        }
    }
    let potential_scheme = &url[..colon_pos];
    let mut chars = potential_scheme.chars();
    match chars.next() {
        Some(first) if first.is_ascii_alphabetic() => {
            if chars.all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '-' || c == '.') {
                Some(potential_scheme)
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Detects the Windows drive-letter quirk (`C:\...`, `C:/...`) where a single
/// ASCII letter immediately followed by `:` and a path separator is a local
/// filesystem path, not a URI scheme. Without this check, `extract_scheme`
/// happily parses `C` out of `C:\Windows\System32` as a syntactically valid
/// (if unrecognized) scheme, which still gets rejected but with a misleading
/// "unsupported protocol" message instead of the correct "absolute local
/// path" one.
fn is_windows_drive_path(url: &str) -> bool {
    let mut chars = url.chars();
    matches!(chars.next(), Some(letter) if letter.is_ascii_alphabetic())
        && chars.next() == Some(':')
        && matches!(chars.next(), Some('/') | Some('\\'))
}

/// Export validates hyperlinks to ensure they use allowed external protocols or safe document-relative paths / fragment anchors.
pub(super) fn validate(url: &str) -> Result<(), String> {
    let trimmed = url.trim();
    if trimmed.is_empty() || trimmed.chars().any(char::is_control) {
        return Err("Document hyperlinks must not be empty or contain control characters".into());
    }

    // Absolute local/UNC filesystem paths must be rejected before generic
    // scheme parsing: both `/...`, `\\...` and Windows drive-letter paths
    // (`C:\...`) are local paths, not URI schemes.
    if trimmed.starts_with('/') || trimmed.starts_with('\\') || is_windows_drive_path(trimmed) {
        return Err("Document hyperlinks must not target absolute local filesystem paths".into());
    }

    if let Some(scheme) = extract_scheme(trimmed) {
        if ["https", "http", "mailto"]
            .iter()
            .any(|allowed| scheme.eq_ignore_ascii_case(allowed))
        {
            Ok(())
        } else {
            Err("Document hyperlinks must use HTTPS, HTTP or mailto; unsafe or unrecognized protocols are not supported".into())
        }
    } else {
        // No scheme: must be a fragment anchor (#...) or safe relative path.
        Ok(())
    }
}
