//! A quote proven to exist in the passage it is attributed to.
//!
//! Models are never trusted to cite. Whatever a model says supports a statement, the support only
//! counts once its quote is found in the passage the Run actually read, at that passage's pinned
//! revision. Text, locator and revision come from the stored passage, never from model output.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Longest quote a span keeps; a longer quote is evidence for too much at once.
pub(crate) const MAX_QUOTE_BYTES: usize = 2_000;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub(crate) struct EvidenceSpan {
    pub chunk_id: String,
    pub node_id: String,
    pub source_revision: String,
    #[ts(type = "unknown")]
    pub locator: Value,
    /// UTF-8 byte offsets of the quote inside the passage text.
    pub quote_start: usize,
    pub quote_end: usize,
    pub quote: String,
}

/// The passage a quote is checked against: one chunk at one revision, as the Run read it.
pub(crate) struct Passage<'a> {
    pub chunk_id: &'a str,
    pub node_id: &'a str,
    pub source_revision: &'a str,
    pub locator: &'a Value,
    pub text: &'a str,
}

impl Passage<'_> {
    /// The span of `quote` in this passage, or nothing when the passage does not contain it.
    pub(crate) fn span(&self, quote: &str) -> Option<EvidenceSpan> {
        let (start, end) = locate(self.text, quote)?;
        Some(EvidenceSpan {
            chunk_id: self.chunk_id.to_owned(),
            node_id: self.node_id.to_owned(),
            source_revision: self.source_revision.to_owned(),
            locator: self.locator.clone(),
            quote_start: start,
            quote_end: end,
            quote: self.text[start..end].to_owned(),
        })
    }
}

/// Byte range of `quote` in `text`: verbatim first, then ignoring case, whitespace runs and
/// typographic variants of quotes and dashes, which models normalize when they copy text. The
/// range always covers original text, so the stored quote is what the source says.
pub(crate) fn locate(text: &str, quote: &str) -> Option<(usize, usize)> {
    let quote = quote.trim();
    if quote.is_empty() || quote.len() > MAX_QUOTE_BYTES {
        return None;
    }
    if let Some(start) = text.find(quote) {
        return Some((start, start + quote.len()));
    }
    let haystack = Folded::new(text);
    let needle = Folded::new(quote).text;
    if needle.is_empty() {
        return None;
    }
    let found = haystack.text.find(&needle)?;
    let start = haystack.origin[found].0;
    let end = haystack.origin[found + needle.len() - 1].1;
    Some((start, end)).filter(|(start, end)| end > start && end - start <= MAX_QUOTE_BYTES * 2)
}

/// Text folded for matching, with the original byte range behind every folded byte.
struct Folded {
    text: String,
    origin: Vec<(usize, usize)>,
}

impl Folded {
    fn new(source: &str) -> Self {
        let mut folded = Self {
            text: String::with_capacity(source.len()),
            origin: Vec::with_capacity(source.len()),
        };
        let mut pending_space: Option<(usize, usize)> = None;
        for (start, character) in source.char_indices() {
            let range = (start, start + character.len_utf8());
            if character.is_whitespace() {
                pending_space.get_or_insert(range);
                continue;
            }
            if let Some(space) = pending_space.take() {
                if !folded.text.is_empty() {
                    folded.push(' ', space);
                }
            }
            for lowered in fold(character).to_lowercase() {
                folded.push(lowered, range);
            }
        }
        folded
    }

    fn push(&mut self, character: char, range: (usize, usize)) {
        self.text.push(character);
        self.origin
            .extend(std::iter::repeat_n(range, character.len_utf8()));
    }
}

fn fold(character: char) -> char {
    match character {
        '\u{2018}' | '\u{2019}' | '\u{201B}' | '\u{2032}' | '`' => '\'',
        '\u{201C}' | '\u{201D}' | '\u{201F}' | '\u{2033}' | '「' | '」' | '『' | '』' => '"',
        '\u{2010}'..='\u{2015}' | '\u{2212}' => '-',
        '\u{2026}' => '.',
        '，' => ',',
        '：' => ':',
        '；' => ';',
        '（' => '(',
        '）' => ')',
        other => other,
    }
}

#[cfg(test)]
#[path = "tests/evidence_span.rs"]
mod tests;
