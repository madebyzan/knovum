/// Formats a millisecond timestamp as ASS `H:MM:SS.cc` (centiseconds).
pub(super) fn timestamp(milliseconds: u64) -> String {
    format!(
        "{}:{:02}:{:02}.{:02}",
        milliseconds / 3_600_000,
        milliseconds / 60_000 % 60,
        milliseconds / 1_000 % 60,
        milliseconds % 1_000 / 10
    )
}

/// Escapes ASS override delimiters and normalizes line breaks to `\N`.
pub(super) fn escape_text(value: &str) -> String {
    let mut escaped = String::new();
    for ch in value.replace("\r\n", "\n").chars() {
        match ch {
            '\n' | '\r' => escaped.push_str("\\N"),
            '{' => escaped.push_str("\\{"),
            '\\' => escaped.push_str("\\\u{2060}"),
            _ => escaped.push(ch),
        }
    }
    escaped
}
