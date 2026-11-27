const MAX_SEGMENT_CHARACTERS: usize = 400;

fn is_non_speakable(character: char) -> bool {
    if character.is_control() && !character.is_whitespace() {
        return true;
    }
    matches!(
        character,
        '\u{00AD}'
            | '\u{200B}'..='\u{200F}'
            | '\u{202A}'..='\u{202E}'
            | '\u{2060}'..='\u{206F}'
            | '\u{20E3}'
            | '\u{2190}'..='\u{21FF}'
            | '\u{2500}'..='\u{25FF}'
            | '\u{2700}'..='\u{27BF}'
            | '\u{FE0E}'
            | '\u{FE0F}'
            | '\u{FEFF}'
            | '\u{1F000}'..='\u{1FAFF}'
    )
}

pub(crate) fn normalize(value: &str) -> String {
    let cleaned: String = value
        .chars()
        .filter(|character| !is_non_speakable(*character))
        .collect();
    cleaned.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub(crate) fn parse_lexicon(value: &serde_json::Value) -> Result<Vec<(String, String)>, String> {
    let entries: std::collections::BTreeMap<String, String> =
        Deserialize::deserialize(value).map_err(crate::spaces::db::json_error)?;
    validate_lexicon(&entries)?;
    let mut entries: Vec<_> = entries.into_iter().collect();
    // Match the longest source first; replacements are never fed back into the lexicon.
    entries.sort_by(|(a, _), (b, _)| b.len().cmp(&a.len()).then_with(|| a.cmp(b)));
    Ok(entries)
}

pub(crate) fn validate_lexicon(
    entries: &std::collections::BTreeMap<String, String>,
) -> Result<(), String> {
    if entries.len() > 1000
        || entries.iter().any(|(word, pronunciation)| {
            word.trim().is_empty()
                || word.chars().count() > 80
                || pronunciation.trim().is_empty()
                || pronunciation.chars().count() > 160
                || word != &normalize(word)
                || pronunciation != &normalize(pronunciation)
        })
    {
        return Err("Pronunciation lexicon entries are invalid".to_owned());
    }
    Ok(())
}

pub(crate) fn apply_lexicon(value: &str, entries: &[(String, String)]) -> String {
    let normalized = normalize(value);
    let mut rest = normalized.as_str();
    let mut output = String::new();
    while !rest.is_empty() {
        if let Some((word, pronunciation)) = entries.iter().find(|(word, _)| rest.starts_with(word))
        {
            output.push_str(pronunciation);
            rest = &rest[word.len()..];
        } else if let Some(character) = rest.chars().next() {
            output.push(character);
            rest = &rest[character.len_utf8()..];
        }
    }
    output
}

pub(super) fn segment(value: &str) -> Result<Vec<String>, String> {
    if value.chars().count() > 1_000_000 {
        return Err("Speech script exceeds one million characters".to_owned());
    }
    let mut segments = Vec::new();
    let mut current = String::new();
    let mut characters = 0;
    let mut soft_boundary = None;
    let mut input = value.chars().peekable();
    while let Some(character) = input.next() {
        if is_non_speakable(character) {
            continue;
        }
        if character.is_whitespace() {
            if !current.ends_with(' ') && !current.is_empty() {
                current.push(' ');
                characters += 1;
            }
        } else {
            current.push(character);
            characters += 1;
        }
        if characters >= MAX_SEGMENT_CHARACTERS / 2
            && (character.is_whitespace()
                || matches!(character, '，' | ',' | '；' | ';' | '：' | ':'))
        {
            soft_boundary = Some(current.len());
        }
        if matches!(character, '。' | '！' | '？' | '!' | '?')
            || (character == '.' && input.peek().is_none_or(|next| next.is_whitespace()))
        {
            push_segment(&mut segments, &mut current);
            characters = 0;
            soft_boundary = None;
        } else if characters >= MAX_SEGMENT_CHARACTERS {
            if let Some(boundary) = soft_boundary {
                let remainder = current.split_off(boundary);
                push_segment(&mut segments, &mut current);
                current = remainder.trim_start().to_owned();
                characters = current.chars().count();
            } else {
                push_segment(&mut segments, &mut current);
                characters = 0;
            }
            soft_boundary = None;
        }
    }
    push_segment(&mut segments, &mut current);
    if segments.is_empty() {
        return Err("Speech script must not be empty".to_owned());
    }
    Ok(segments)
}

fn push_segment(segments: &mut Vec<String>, current: &mut String) {
    let text = current.trim();
    if !text.is_empty() && text.chars().any(char::is_alphanumeric) {
        segments.push(text.to_owned());
    }
    current.clear();
}
use serde::Deserialize;
