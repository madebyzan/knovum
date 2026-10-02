const ASSET_URL_PREFIXES: [&str; 3] = ["asset://localhost/", "http://asset.localhost/", "asset://"];

pub(super) fn managed_path(destination: &str) -> Option<String> {
    if ASSET_URL_PREFIXES
        .iter()
        .any(|prefix| destination.starts_with(prefix))
    {
        return Some(asset_path(destination));
    }
    if destination.starts_with(super::PACKAGE_ASSET_PREFIX) || destination.starts_with('/') {
        return Some(destination.to_owned());
    }
    let bytes = destination.as_bytes();
    (bytes.len() > 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':' && bytes[2] == b'\\')
        .then(|| destination.to_owned())
}

pub(super) fn asset_path(destination: &str) -> String {
    for prefix in ASSET_URL_PREFIXES {
        if let Some(rest) = destination.strip_prefix(prefix) {
            return percent_decode(rest);
        }
    }
    destination.to_owned()
}

fn percent_decode(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut cursor = 0;
    while cursor < bytes.len() {
        let byte = bytes[cursor];
        if byte == b'%' {
            if let Some(value) = escape_at(bytes, cursor) {
                decoded.push(value);
                cursor += 3;
                continue;
            }
        }
        decoded.push(byte);
        cursor += 1;
    }
    String::from_utf8_lossy(&decoded).into_owned()
}

/// A `%XX` escape is only consumed when both trailing bytes are hex digits; a
/// literal `%` must survive so the decoded value still names the file on disk.
fn escape_at(bytes: &[u8], cursor: usize) -> Option<u8> {
    let high = hex_value(*bytes.get(cursor + 1)?)?;
    let low = hex_value(*bytes.get(cursor + 2)?)?;
    Some(high * 16 + low)
}

fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}
