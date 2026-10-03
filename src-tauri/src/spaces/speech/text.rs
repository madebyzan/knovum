const MAX_SEGMENT_CHARACTERS: usize = 400;

pub(crate) fn normalize(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub(super) fn segment(value: &str) -> Result<Vec<String>, String> {
    if value.chars().count() > 1_000_000 {
        return Err("Speech script exceeds one million characters".to_owned());
    }
    let mut segments = Vec::new();
    let mut current = String::new();
    let mut characters = 0;
    let mut soft_boundary = None;
    for character in value.chars() {
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
            && matches!(character, '，' | ',' | '；' | ';' | '：' | ':' | '\n')
        {
            soft_boundary = Some(current.len());
        }
        if matches!(character, '。' | '！' | '？' | '.' | '!' | '?') {
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
    if !text.is_empty() {
        segments.push(text.to_owned());
    }
    current.clear();
}
