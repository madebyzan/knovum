use std::collections::HashMap;
use std::fmt::Write;

use serde_json::Value;

/// Strips internal `[Cn]` citation anchors outside fenced code blocks and inline code spans,
/// cleaning up horizontal whitespace around removed markers so exported prose reads naturally.
pub(crate) fn strip_citation_markers(text: &str) -> String {
    let mut remap = HashMap::new();
    let mut ordered = Vec::new();
    rewrite_citation_markers(text, 0, &mut remap, &mut ordered)
}

/// Rewrites `[Cn]` markers in `answer` to compact sequential `[C1]..[Ck]` in order of first
/// appearance, drops out-of-range markers outside code spans/fences, and builds the matching
/// Grounded citation records.
pub(crate) fn finalize_citations(
    answer: &str,
    sources: &[Value],
    current_space_id: &str,
) -> Result<(String, Value), String> {
    let mut remap: HashMap<usize, usize> = HashMap::new();
    let mut ordered_sources: Vec<(usize, usize)> = Vec::new();
    let rewritten = rewrite_citation_markers(answer, sources.len(), &mut remap, &mut ordered_sources);
    let mut citations = Vec::with_capacity(ordered_sources.len());
    for (compact_index, original_index) in ordered_sources {
        let Some(source) = original_index
            .checked_sub(1)
            .and_then(|position| sources.get(position))
        else {
            continue;
        };
        let external = source
            .get("spaceId")
            .and_then(Value::as_str)
            .is_some_and(|space_id| space_id != current_space_id);
        let mut citation = serde_json::json!({
            "index": compact_index,
            "state": if external { "detached" } else { "current" },
            "nodeTitle": source["nodeTitle"],
            "sourceRevision": source["sourceRevision"],
            "locator": source["locator"],
            "quote": source["text"].as_str().unwrap_or(""),
        });
        let object = citation
            .as_object_mut()
            .ok_or_else(|| "Failed to format citation object".to_owned())?;
        if external {
            if let Some(title) = source.get("spaceTitle") {
                object.insert("sourceSpaceTitle".to_owned(), title.clone());
            }
        } else {
            object.insert("chunkId".to_owned(), source["chunkId"].clone());
            object.insert("nodeId".to_owned(), source["nodeId"].clone());
        }
        citations.push(citation);
    }
    Ok((rewritten, Value::Array(citations)))
}

fn rewrite_citation_markers(
    input: &str,
    source_count: usize,
    remap: &mut HashMap<usize, usize>,
    ordered: &mut Vec<(usize, usize)>,
) -> String {
    let mut output = String::with_capacity(input.len());
    let mut in_fence: Option<(char, usize)> = None;
    for segment in input.split_inclusive('\n') {
        let trimmed = segment.trim_start();
        if let Some((fence_char, fence_len)) = in_fence {
            output.push_str(segment);
            if is_closing_fence(trimmed, fence_char, fence_len) {
                in_fence = None;
            }
            continue;
        }
        if let Some(fence) = opening_fence(trimmed) {
            in_fence = Some(fence);
            output.push_str(segment);
            continue;
        }
        rewrite_line(segment, source_count, remap, ordered, &mut output);
    }
    output
}

fn rewrite_line(
    line: &str,
    source_count: usize,
    remap: &mut HashMap<usize, usize>,
    ordered: &mut Vec<(usize, usize)>,
    output: &mut String,
) {
    let bytes = line.as_bytes();
    let mut cursor = 0;
    let mut last_emitted: Option<usize> = None;
    let mut seen_in_group: Vec<usize> = Vec::new();
    while cursor < bytes.len() {
        if bytes[cursor] == b'`' {
            last_emitted = None;
            let ticks = count_byte(bytes, cursor, b'`');
            let rest = &line[cursor + ticks..];
            if let Some(close_offset) = find_closing_backticks(rest, ticks) {
                let end = cursor + ticks + close_offset + ticks;
                output.push_str(&line[cursor..end]);
                cursor = end;
            } else {
                output.push_str(&line[cursor..cursor + ticks]);
                cursor += ticks;
            }
            continue;
        }
        if bytes[cursor] == b'[' {
            if let Some((consumed, indices)) = parse_citation_bracket(&line[cursor..]) {
                let mut emitted_any = false;
                seen_in_group.clear();
                for original in indices {
                    if original == 0 || original > source_count {
                        continue;
                    }
                    let compact = *remap.entry(original).or_insert_with(|| {
                        let next = ordered.len() + 1;
                        ordered.push((next, original));
                        next
                    });
                    if last_emitted == Some(compact) || seen_in_group.contains(&compact) {
                        emitted_any = true;
                        continue;
                    }
                    seen_in_group.push(compact);
                    let _ = write!(output, "[C{compact}]");
                    last_emitted = Some(compact);
                    emitted_any = true;
                }
                if !emitted_any && should_trim_space_before(&line[cursor + consumed..]) {
                    trim_trailing_spaces(output);
                }
                cursor += consumed;
                continue;
            }
        }
        let ch = line[cursor..].chars().next().unwrap();
        output.push(ch);
        if !ch.is_ascii_whitespace() {
            last_emitted = None;
        }
        cursor += ch.len_utf8();
    }
}

/// Parses `[C5]` or comma-grouped `[C5, C12]` at the start of `slice`, returning `(bytes_consumed, indices)`.
fn parse_citation_bracket(slice: &str) -> Option<(usize, Vec<usize>)> {
    if slice.as_bytes().first() != Some(&b'[') {
        return None;
    }
    let close = slice.find(']')?;
    let inner = &slice[1..close];
    if inner.is_empty() {
        return None;
    }
    let mut indices = Vec::new();
    for part in inner.split(',') {
        let digits = part.trim().strip_prefix('C')?;
        if digits.is_empty() || !digits.bytes().all(|byte| byte.is_ascii_digit()) {
            return None;
        }
        indices.push(digits.parse::<usize>().ok()?);
    }
    (!indices.is_empty()).then_some((close + 1, indices))
}

fn opening_fence(trimmed: &str) -> Option<(char, usize)> {
    let first = trimmed.chars().next()?;
    if first != '`' && first != '~' {
        return None;
    }
    let len = trimmed.chars().take_while(|&ch| ch == first).count();
    (len >= 3).then_some((first, len))
}

fn is_closing_fence(trimmed: &str, fence_char: char, min_len: usize) -> bool {
    let len = trimmed.chars().take_while(|&ch| ch == fence_char).count();
    len >= min_len && trimmed[len * fence_char.len_utf8()..].trim().is_empty()
}

fn count_byte(bytes: &[u8], start: usize, target: u8) -> usize {
    bytes[start..].iter().take_while(|&&byte| byte == target).count()
}

fn find_closing_backticks(rest: &str, ticks: usize) -> Option<usize> {
    let bytes = rest.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'`' {
            let run = count_byte(bytes, index, b'`');
            if run == ticks {
                return Some(index);
            }
            index += run;
        } else {
            index += 1;
        }
    }
    None
}

fn should_trim_space_before(rest: &str) -> bool {
    matches!(
        rest.chars().next(),
        None | Some(
            ' ' | '\t'
                | '\n'
                | '\r'
                | '.'
                | ','
                | ';'
                | ':'
                | '!'
                | '?'
                | '。'
                | '，'
                | '；'
                | '：'
                | '！'
                | '？'
                | '、'
        )
    )
}

fn trim_trailing_spaces(buffer: &mut String) {
    while buffer.ends_with(' ') || buffer.ends_with('\t') {
        buffer.pop();
    }
}
