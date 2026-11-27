/// Preprocesses Knovum-authored Markdown before `pulldown-cmark` parsing:
/// - Strips standalone `&nbsp;` / U+00A0 empty-paragraph placeholders outside fenced code blocks.
/// - Unwraps `:::gallery{...}` / `:::` directive fences so gallery images export as standard
///   images without leaking raw directive syntax into HTML, DOCX, PDF, PPTX, or TXT.
pub(crate) fn clean(markdown: &str) -> String {
    let mut output = String::with_capacity(markdown.len());
    let mut fence: Option<(char, usize)> = None;
    let mut in_gallery = false;
    let mut first = true;

    for line in markdown.split('\n') {
        let trimmed_cr = line.strip_suffix('\r').unwrap_or(line);
        let trimmed = trimmed_cr.trim();

        if let Some((fence_char, fence_len)) = fence {
            if is_closing_fence(trimmed, fence_char, fence_len) {
                fence = None;
            }
            push_line(&mut output, trimmed_cr, &mut first);
            continue;
        }

        if let Some(opened) = parse_opening_fence(trimmed_cr) {
            fence = Some(opened);
            push_line(&mut output, trimmed_cr, &mut first);
            continue;
        }

        if is_gallery_open(trimmed) {
            in_gallery = true;
            continue;
        }
        if in_gallery && trimmed == ":::" {
            in_gallery = false;
            continue;
        }
        if is_nbsp_only_line(trimmed) {
            push_line(&mut output, "", &mut first);
            continue;
        }

        push_line(&mut output, trimmed_cr, &mut first);
    }

    output
}

fn push_line(output: &mut String, line: &str, first: &mut bool) {
    if !*first {
        output.push('\n');
    }
    *first = false;
    output.push_str(line);
}

fn parse_opening_fence(line: &str) -> Option<(char, usize)> {
    let indent = line.bytes().take_while(|&b| b == b' ').count();
    if indent > 3 {
        return None;
    }
    let rest = &line[indent..];
    let first = rest.chars().next()?;
    if first != '`' && first != '~' {
        return None;
    }
    let count = rest.chars().take_while(|&c| c == first).count();
    if count < 3 {
        return None;
    }
    if first == '`' && rest[count..].contains('`') {
        return None;
    }
    Some((first, count))
}

fn is_closing_fence(trimmed: &str, fence_char: char, fence_len: usize) -> bool {
    let count = trimmed.chars().take_while(|&c| c == fence_char).count();
    count >= fence_len && trimmed.chars().all(|c| c == fence_char)
}

fn is_gallery_open(trimmed: &str) -> bool {
    let Some(rest) = trimmed.strip_prefix(":::gallery") else {
        return false;
    };
    rest.is_empty() || (rest.starts_with('{') && rest.ends_with('}'))
}

fn is_nbsp_only_line(trimmed: &str) -> bool {
    if trimmed.is_empty() {
        return false;
    }
    let mut rest = trimmed;
    while !rest.is_empty() {
        if let Some(after) = rest.strip_prefix("&nbsp;") {
            rest = after.trim_start();
        } else if let Some(after) = rest.strip_prefix('\u{a0}') {
            rest = after.trim_start();
        } else {
            return false;
        }
    }
    true
}
