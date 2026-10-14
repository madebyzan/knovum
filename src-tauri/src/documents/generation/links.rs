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

/// Export validates hyperlinks to ensure they use allowed external protocols or safe document-relative paths / fragment anchors.
pub(super) fn validate(url: &str) -> Result<(), String> {
    let trimmed = url.trim();
    if trimmed.is_empty() || trimmed.chars().any(char::is_control) {
        return Err("Document hyperlinks must not be empty or contain control characters".into());
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
        // Disallow absolute local/UNC filesystem paths (/... or \\...).
        if trimmed.starts_with('/') || trimmed.starts_with('\\') {
            return Err(
                "Document hyperlinks must not target absolute local filesystem paths".into(),
            );
        }
        Ok(())
    }
}
