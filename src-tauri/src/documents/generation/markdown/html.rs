use super::super::document::Inline;

/// Elements whose text is not prose. Their content says nothing about the document, so neither
/// the tags nor what they wrap reach the model.
const SILENT: [&str; 6] = ["script", "style", "svg", "iframe", "noscript", "template"];

/// Reads the raw HTML a Markdown document carries.
///
/// A document is the content markup wraps, not the markup: text belongs to the document and
/// `<br>` is a line break, while a tag that says nothing about content contributes nothing. This
/// is what makes ordinary Markdown - a badge row, a centred header, a `<sub>` caption -
/// convertible instead of refused.
pub(super) fn read(html: &str, output: &mut Vec<Inline>) {
    let mut rest = html;
    let mut silent: Option<String> = None;
    while !rest.is_empty() {
        let Some(start) = rest.find('<') else {
            push_text(rest, silent.is_some(), output);
            return;
        };
        push_text(&rest[..start], silent.is_some(), output);
        rest = &rest[start..];
        if let Some(comment) = rest.strip_prefix("<!--") {
            // An unterminated comment comments out the remainder.
            rest = comment
                .find("-->")
                .map_or("", |end| &comment[end + "-->".len()..]);
            continue;
        }
        let Some(end) = rest.find('>') else {
            // An unterminated tag is markup all the way to the end, so it wraps nothing.
            return;
        };
        let tag = &rest[1..end];
        rest = &rest[end + 1..];
        let closing = tag.starts_with('/');
        let name = name(tag);
        // Inside an element whose text is not prose, nothing but its own closing tag matters.
        if let Some(open) = silent.take() {
            if !closing || name != open {
                silent = Some(open);
            }
            continue;
        }
        if SILENT.contains(&name.as_str()) {
            if !closing && !tag.trim_end().ends_with('/') {
                silent = Some(name);
            }
            continue;
        }
        if closing {
            continue;
        }
        match name.as_str() {
            "br" => output.push(Inline::HardBreak),
            // Bytes are embedded from pictures this Space holds, which are the ones a document
            // references in Markdown. An HTML picture says what it shows, and that is its text.
            "img" => push_text(&attribute(tag, "alt").unwrap_or_default(), false, output),
            _ => {}
        }
    }
}

/// HTML text, as the document reads it: entities resolved and runs of ASCII whitespace collapsed,
/// the way every HTML renderer treats them - a no-break space is content, not spacing. A run that
/// is only whitespace separates what surrounds it, and is dropped where there is nothing to
/// separate.
fn push_text(value: &str, silent: bool, output: &mut Vec<Inline>) {
    if silent {
        return;
    }
    let text = collapsed(&decoded(value));
    if text.is_empty() || (text == " " && output.is_empty()) {
        return;
    }
    output.push(Inline::Text(text));
}

fn collapsed(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    let mut space = false;
    for character in value.chars() {
        if character.is_ascii_whitespace() {
            space = true;
            continue;
        }
        if space && !output.is_empty() {
            output.push(' ');
        }
        space = false;
        output.push(character);
    }
    if space && !output.is_empty() {
        output.push(' ');
    }
    output
}

fn decoded(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    let mut rest = value;
    while let Some(start) = rest.find('&') {
        output.push_str(&rest[..start]);
        rest = &rest[start..];
        let end = rest[1..].find(';').map(|end| end + 1);
        match end.and_then(|end| character(&rest[1..end]).map(|value| (value, end))) {
            Some((character, end)) => {
                output.push(character);
                rest = &rest[end + 1..];
            }
            None => {
                output.push('&');
                rest = &rest[1..];
            }
        }
    }
    output.push_str(rest);
    output
}

/// The named references HTML text relies on, plus the numeric form that names the rest.
fn character(reference: &str) -> Option<char> {
    match reference {
        "amp" => Some('&'),
        "lt" => Some('<'),
        "gt" => Some('>'),
        "quot" => Some('"'),
        "apos" => Some('\''),
        "nbsp" => Some('\u{a0}'),
        reference => {
            let digits = reference.strip_prefix('#')?;
            let code = match digits.strip_prefix(['x', 'X']) {
                Some(hexadecimal) => u32::from_str_radix(hexadecimal, 16).ok()?,
                None => digits.parse().ok()?,
            };
            char::from_u32(code)
        }
    }
}

fn name(tag: &str) -> String {
    tag.trim_start_matches('/')
        .split(|character: char| character.is_whitespace() || character == '/')
        .next()
        .unwrap_or_default()
        .to_ascii_lowercase()
}

/// One attribute of a tag, as HTML spells them: quoted, single-quoted or bare.
fn attribute(tag: &str, name: &str) -> Option<String> {
    // Lowercasing ASCII preserves every byte position, so the lowered copy locates names in the
    // original without shifting its values.
    let lowered = tag.to_ascii_lowercase();
    let mut from = 0;
    while let Some(found) = lowered[from..].find(name) {
        let at = from + found;
        from = at + name.len();
        let separated = lowered[..at]
            .chars()
            .next_back()
            .is_none_or(char::is_whitespace);
        if separated && lowered[from..].trim_start().starts_with('=') {
            return Some(value_of(tag[from..].trim_start()));
        }
    }
    None
}

/// The value of `= value`, where the caller has already seen the `=`.
fn value_of(assignment: &str) -> String {
    let value = assignment["=".len()..].trim_start();
    let quote = value
        .chars()
        .next()
        .filter(|first| *first == '"' || *first == '\'');
    let rest = if quote.is_some() { &value[1..] } else { value };
    let end = match quote {
        Some(quote) => rest.find(quote),
        None => rest.find(char::is_whitespace),
    };
    decoded(&rest[..end.unwrap_or(rest.len())])
        .trim()
        .to_owned()
}
