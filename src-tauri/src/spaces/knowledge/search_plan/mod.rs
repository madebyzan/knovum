const MAX_OR_BRANCHES: usize = 6;
const MAX_TERMS_PER_BRANCH: usize = 8;

#[derive(Debug, Clone)]
pub(crate) struct KnowledgeSearchPlan {
    /// Flattened unique searchable terms across all branches (for highlighting and legacy callers).
    pub terms: Vec<String>,
    /// Disjunctive normal form: matches when ANY branch matches ALL of its terms.
    pub branches: Vec<Vec<String>>,
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
    let mut terms = Vec::new();
    let mut branches = Vec::new();
    let mut fts_branches = Vec::new();
    let mut all_branches_fts_eligible = true;

    for raw_branch in value.split('|').take(MAX_OR_BRANCHES) {
        let raw_tokens = raw_branch
            .split_whitespace()
            .map(clean_token)
            .filter(|token| !token.is_empty())
            .collect::<Vec<_>>();
        if raw_tokens.is_empty() {
            continue;
        }
        let branch_terms = expand_branch_terms(&raw_tokens);
        if branch_terms.is_empty() {
            continue;
        }
        for term in &branch_terms {
            if !terms.contains(term) {
                terms.push(term.clone());
            }
        }
        if raw_tokens.iter().any(|token| token.chars().count() < 3) {
            all_branches_fts_eligible = false;
        } else {
            let clause = raw_tokens
                .iter()
                .take(MAX_TERMS_PER_BRANCH)
                .map(|token| format!("\"{}\"", token.replace('"', "\"\"")))
                .collect::<Vec<_>>()
                .join(" AND ");
            fts_branches.push(clause);
        }
        branches.push(branch_terms);
    }

    if branches.is_empty() {
        return Ok(KnowledgeSearchPlan {
            terms: Vec::new(),
            branches: Vec::new(),
            fts_query: None,
        });
    }
    terms.truncate(MAX_OR_BRANCHES * MAX_TERMS_PER_BRANCH);
    let fts_query = if all_branches_fts_eligible && !fts_branches.is_empty() {
        Some(if fts_branches.len() == 1 {
            fts_branches.into_iter().next().unwrap_or_default()
        } else {
            fts_branches
                .into_iter()
                .map(|clause| format!("({clause})"))
                .collect::<Vec<_>>()
                .join(" OR ")
        })
    } else {
        None
    };
    Ok(KnowledgeSearchPlan {
        terms,
        branches,
        fts_query,
    })
}

fn expand_branch_terms(raw_tokens: &[String]) -> Vec<String> {
    let mut branch_terms = Vec::new();
    for token in raw_tokens {
        let chars: Vec<char> = token.chars().collect();
        let is_cjk = chars.iter().any(|&ch| is_cjk_char(ch));
        if chars.len() >= 4 && is_cjk {
            let mut index = 0;
            while index < chars.len() {
                let remaining = chars.len() - index;
                let chunk_size = if remaining == 3 { 3 } else { 2.min(remaining) };
                let subterm: String = chars[index..index + chunk_size].iter().collect();
                if !branch_terms.contains(&subterm) {
                    branch_terms.push(subterm);
                }
                index += chunk_size;
            }
        } else if !branch_terms.contains(token) {
            branch_terms.push(token.clone());
        }
    }
    branch_terms.truncate(MAX_TERMS_PER_BRANCH);
    branch_terms
}

fn clean_token(value: &str) -> String {
    value
        .trim_matches(|character| matches!(character, '"' | '\'' | '“' | '”' | '‘' | '’'))
        .trim()
        .to_owned()
}

#[cfg(test)]
mod tests;
