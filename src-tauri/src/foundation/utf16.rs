pub(crate) fn truncate_with_suffix(text: &str, limit: usize, suffix: &str) -> String {
    if text.encode_utf16().count() <= limit {
        return text.to_owned();
    }
    let suffix_width = suffix.encode_utf16().count();
    if suffix_width > limit {
        return String::new();
    }
    let budget = limit - suffix_width;
    let mut used = 0usize;
    let mut end = 0usize;
    for (index, character) in text.char_indices() {
        let width = character.len_utf16();
        if used + width > budget {
            break;
        }
        used += width;
        end = index + character.len_utf8();
    }
    let mut output = text[..end].to_owned();
    output.push_str(suffix);
    output
}
