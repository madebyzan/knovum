/// Builds a compact, match-centered excerpt from `text` so CJK substring and semantic
/// search hits match FTS5 snippet density instead of dumping full multi-paragraph chunks.
pub(super) fn build_excerpt(text: &str, terms: &[String], max_chars: usize) -> String {
    let collapsed = text.split_whitespace().collect::<Vec<_>>().join(" ");
    let chars: Vec<char> = collapsed.chars().collect();
    if chars.len() <= max_chars {
        return collapsed;
    }

    let match_char_index =
        find_earliest_match_char(&collapsed, terms).map(|index| index.min(chars.len()));
    let lead_in = max_chars / 3;
    let mut start = match_char_index
        .map(|index| index.saturating_sub(lead_in))
        .unwrap_or(0);
    if start + max_chars > chars.len() {
        start = chars.len().saturating_sub(max_chars);
    }
    if start > 0 {
        start = snap_start_boundary(&chars, start);
    }
    let mut end = (start + max_chars).min(chars.len());
    if end < chars.len() {
        end = snap_end_boundary(&chars, start, end);
    }

    let slice: String = chars[start..end].iter().collect();
    let trimmed = slice.trim();
    match (start > 0, end < chars.len()) {
        (true, true) => format!("… {trimmed} …"),
        (true, false) => format!("… {trimmed}"),
        (false, true) => format!("{trimmed} …"),
        (false, false) => trimmed.to_owned(),
    }
}

fn find_earliest_match_char(collapsed: &str, terms: &[String]) -> Option<usize> {
    let lowered = collapsed.to_lowercase();
    let mut earliest_byte = None;
    for term in terms {
        let needle = term.trim().to_lowercase();
        if needle.is_empty() {
            continue;
        }
        if let Some(byte_index) = lowered.find(&needle) {
            earliest_byte = Some(match earliest_byte {
                Some(current) if current <= byte_index => current,
                _ => byte_index,
            });
        }
    }
    earliest_byte.map(|byte_index| lowered[..byte_index].chars().count())
}

fn snap_start_boundary(chars: &[char], start: usize) -> usize {
    let lookahead_end = (start + 20).min(chars.len());
    for index in start..lookahead_end {
        if chars[index].is_whitespace() {
            return (index + 1).min(chars.len());
        }
    }
    start
}

fn snap_end_boundary(chars: &[char], start: usize, end: usize) -> usize {
    let lookback_start = end.saturating_sub(20).max(start + 1);
    for index in (lookback_start..end).rev() {
        if chars[index].is_whitespace() {
            return index;
        }
    }
    end
}
