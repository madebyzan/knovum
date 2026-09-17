pub(crate) fn parse_progress(line: &str, marker: &str) -> Option<u8> {
    let remainder = if marker.ends_with(' ') {
        line.strip_prefix(marker)?
    } else {
        line.split_once(marker)?.1
    };
    let raw = remainder.trim().strip_suffix('%')?.trim();
    let percent = raw.parse::<f32>().ok()?;
    Some(percent.clamp(0.0, 100.0).round() as u8)
}
