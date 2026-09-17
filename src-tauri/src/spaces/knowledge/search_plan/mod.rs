#[derive(Debug, Clone)]
pub(crate) struct KnowledgeSearchPlan {
    pub terms: Vec<String>,
    pub fts_query: Option<String>,
}

fn is_cjk_char(character: char) -> bool {
    matches!(
        character,
        '\u{4E00}'..='\u{9FFF}'
            | '\u{3400}'..='\u{4DBF}'
            | '\u{F900}'..='\u{FAFF}'
            | '\u{3040}'..='\u{30FF}'
            | '\u{AC00}'..='\u{D7AF}'
    )
}

pub(crate) fn build(value: &str) -> Result<KnowledgeSearchPlan, String> {
    if value.encode_utf16().count() > 4_096 {
        return Err("Knowledge search query is too long".into());
    }
    let raw_tokens = value
        .split_whitespace()
        .map(clean_token)
        .filter(|token| !token.is_empty())
        .collect::<Vec<_>>();

    if raw_tokens.is_empty() {
        return Ok(KnowledgeSearchPlan {
            terms: Vec::new(),
            fts_query: None,
        });
    }

    let mut terms = Vec::new();
    for token in &raw_tokens {
        let chars: Vec<char> = token.chars().collect();
        let is_cjk = chars.iter().any(|&ch| is_cjk_char(ch));
        if chars.len() >= 4 && is_cjk {
            let mut i = 0;
            while i < chars.len() {
                let remaining = chars.len() - i;
                let chunk_size = if remaining == 3 { 3 } else { 2.min(remaining) };
                let subterm: String = chars[i..i + chunk_size].iter().collect();
                if !terms.contains(&subterm) {
                    terms.push(subterm);
                }
                i += chunk_size;
            }
        } else if !terms.contains(token) {
            terms.push(token.clone());
        }
    }
    terms.truncate(8);

    let fts_query = if raw_tokens.iter().any(|term| term.chars().count() < 3) {
        None
    } else {
        Some(
            raw_tokens
                .iter()
                .take(8)
                .map(|term| format!("\"{}\"", term.replace('"', "\"\"")))
                .collect::<Vec<_>>()
                .join(" AND "),
        )
    };
    Ok(KnowledgeSearchPlan { terms, fts_query })
}

fn clean_token(value: &str) -> String {
    value
        .trim_matches(|character| matches!(character, '"' | '\'' | '“' | '”' | '‘' | '’'))
        .trim()
        .to_owned()
}

#[cfg(test)]
mod tests;
