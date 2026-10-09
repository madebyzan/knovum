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
            return percent_encoding::percent_decode_str(rest)
                .decode_utf8_lossy()
                .into_owned();
        }
    }
    destination.to_owned()
}
