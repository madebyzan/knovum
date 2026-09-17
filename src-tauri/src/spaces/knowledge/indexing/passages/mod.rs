use serde_json::Value;

use super::{KnowledgePassage, MAX_KNOWLEDGE_PASSAGE_BYTES};

pub fn split_passages_with_offsets(text: &str) -> Vec<KnowledgePassage> {
    let mut passages = Vec::new();
    let mut cursor = 0usize;
    let mut cursor_utf16 = 0usize;
    while cursor < text.len() {
        while cursor < text.len() {
            let ch = text[cursor..]
                .chars()
                .next()
                .expect("cursor is a char boundary");
            if !ch.is_whitespace() {
                break;
            }
            cursor += ch.len_utf8();
            cursor_utf16 += ch.len_utf16();
        }
        if cursor >= text.len() {
            break;
        }

        let remaining = &text[cursor..];
        let break_at = if remaining.len() <= MAX_KNOWLEDGE_PASSAGE_BYTES {
            remaining.len()
        } else {
            let hard_end = byte_boundary_at_or_before(remaining, MAX_KNOWLEDGE_PASSAGE_BYTES);
            let preferred_start = hard_end.saturating_mul(3) / 5;
            remaining[..hard_end]
                .char_indices()
                .rev()
                .find_map(|(index, ch)| {
                    (index >= preferred_start && ch.is_whitespace()).then_some(index)
                })
                .filter(|index| *index > 0)
                .unwrap_or(hard_end)
        };
        let raw = &remaining[..break_at];
        let part = raw.trim_end();
        if !part.is_empty() {
            let width = part.encode_utf16().count();
            passages.push(KnowledgePassage {
                text: part.to_owned(),
                start_offset: cursor_utf16,
                end_offset: cursor_utf16 + width,
            });
        }
        cursor += break_at;
        cursor_utf16 += raw.encode_utf16().count();
    }
    passages
}

pub fn location_for_passage(location: &Value, start_offset: usize, end_offset: usize) -> Value {
    const JS_MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;
    let mut next = location.clone();
    let Some(object) = next.as_object_mut() else {
        return next;
    };
    let Some(base) = object.get("startOffset").and_then(Value::as_u64) else {
        return next;
    };
    let Some(start) = base.checked_add(start_offset as u64) else {
        return next;
    };
    let Some(end) = base.checked_add(end_offset as u64) else {
        return next;
    };
    if start > JS_MAX_SAFE_INTEGER || end > JS_MAX_SAFE_INTEGER {
        return next;
    }
    object.insert("startOffset".into(), Value::from(start));
    object.insert("endOffset".into(), Value::from(end));
    next
}

fn byte_boundary_at_or_before(value: &str, max: usize) -> usize {
    if value.len() <= max {
        return value.len();
    }
    let mut end = max;
    while end > 0 && !value.is_char_boundary(end) {
        end -= 1;
    }
    end.max(1)
}

#[cfg(test)]
mod tests;
