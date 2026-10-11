use std::{fs, fs::File, io::Read, path::Path, time::UNIX_EPOCH};

const MAX_TEXT_PREVIEW_BYTES: usize = 192 * 1024;
const MAX_TEXT_PREVIEW_LINES: usize = 320;
const MAX_HTML_READ_BYTES: usize = 2 * 1024 * 1024;
const MAX_HTML_PREVIEW_UTF16: usize = 512 * 1024;

pub fn modified_millis(metadata: &fs::Metadata) -> Result<u64, String> {
    let modified = metadata.modified().map_err(|error| error.to_string())?;
    let duration = modified
        .duration_since(UNIX_EPOCH)
        .map_err(|error| error.to_string())?;
    Ok(duration.as_millis().min(u128::from(u64::MAX)) as u64)
}

pub fn read_text_preview(path: &Path) -> Result<String, String> {
    let file = File::open(path).map_err(|error| error.to_string())?;
    let mut bytes = Vec::with_capacity(MAX_TEXT_PREVIEW_BYTES.min(64 * 1024));
    file.take(MAX_TEXT_PREVIEW_BYTES as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| error.to_string())?;
    let truncated_bytes = bytes.len() > MAX_TEXT_PREVIEW_BYTES;
    bytes.truncate(MAX_TEXT_PREVIEW_BYTES);
    if truncated_bytes {
        trim_incomplete_utf8_tail(&mut bytes);
    }
    let text = String::from_utf8_lossy(&bytes);
    let mut lines = text.lines();
    let mut preview = lines
        .by_ref()
        .take(MAX_TEXT_PREVIEW_LINES)
        .collect::<Vec<_>>()
        .join("\n");
    if truncated_bytes || lines.next().is_some() {
        preview.push_str("\n…");
    }
    Ok(preview)
}

pub fn read_html_preview(path: &Path) -> Result<String, String> {
    let file = File::open(path).map_err(|error| error.to_string())?;
    let mut bytes = Vec::with_capacity(MAX_HTML_READ_BYTES.min(128 * 1024));
    file.take(MAX_HTML_READ_BYTES as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| error.to_string())?;
    let truncated = bytes.len() > MAX_HTML_READ_BYTES;
    bytes.truncate(MAX_HTML_READ_BYTES);
    if truncated {
        trim_incomplete_utf8_tail(&mut bytes);
    }
    let preview = String::from_utf8_lossy(&bytes).into_owned();
    let persistence_truncated = preview.encode_utf16().count() > MAX_HTML_PREVIEW_UTF16;
    let suffix = if truncated || persistence_truncated {
        "\n<!-- Knovum preview truncated -->"
    } else {
        ""
    };
    Ok(crate::foundation::utf16::truncate_with_suffix(
        &preview,
        MAX_HTML_PREVIEW_UTF16,
        suffix,
    ))
}

fn trim_incomplete_utf8_tail(bytes: &mut Vec<u8>) {
    if bytes.is_empty() {
        return;
    }
    let mut start = bytes.len() - 1;
    let mut continuations = 0usize;
    while start > 0 && bytes[start] & 0b1100_0000 == 0b1000_0000 && continuations < 3 {
        continuations += 1;
        start -= 1;
    }
    let lead = bytes[start];
    let expected = match lead {
        0xC2..=0xDF => 2,
        0xE0..=0xEF => 3,
        0xF0..=0xF4 => 4,
        _ => return,
    };
    if bytes.len() - start < expected {
        bytes.truncate(start);
    }
}

#[cfg(test)]
mod tests;
