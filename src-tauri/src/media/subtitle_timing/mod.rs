const SILENCE_BOUNDARY_MS: u64 = 650;
const MAX_CUE_DURATION_MS: u64 = 5_000;
const SOFT_LATIN_CHARS: usize = 38;
const HARD_LATIN_CHARS: usize = 54;
const SOFT_CJK_CHARS: usize = 16;
const HARD_CJK_CHARS: usize = 24;

#[cfg(test)]
mod tests;

pub(crate) struct TimedToken {
    pub start_ms: u64,
    pub end_ms: u64,
    pub text: String,
}

pub struct SubtitleCue {
    pub start_ms: u64,
    pub end_ms: u64,
    pub text: String,
}

pub fn cues(tokens: impl IntoIterator<Item = TimedToken>) -> Vec<SubtitleCue> {
    let mut output = Vec::new();
    let mut active = ActiveCue::default();
    for token in tokens {
        if !valid(&token) {
            continue;
        }
        if active.should_end_before(&token) {
            active.finish(&mut output);
        }
        active.push(token);
        if active.ends_sentence() || active.should_end_after_clause() {
            active.finish(&mut output);
        }
    }
    active.finish(&mut output);
    output
}

fn valid(token: &TimedToken) -> bool {
    token.end_ms > token.start_ms && !token.text.trim().is_empty()
}

pub(crate) fn is_cjk(ch: char) -> bool {
    matches!(
        ch as u32,
        0x3040..=0x30FF | 0x3400..=0x4DBF | 0x4E00..=0x9FFF | 0xAC00..=0xD7AF | 0xF900..=0xFAFF
    )
}

fn is_opening_punct(ch: char) -> bool {
    matches!(
        ch,
        '(' | '[' | '{' | '"' | '\'' | '“' | '‘' | '/' | '-' | '—' | '（' | '【' | '《'
    )
}

fn is_attaching_punct(ch: char) -> bool {
    matches!(
        ch,
        ',' | '.'
            | '!'
            | '?'
            | ';'
            | ':'
            | ')'
            | ']'
            | '}'
            | '\''
            | '’'
            | '"'
            | '”'
            | '…'
            | '、'
            | '。'
            | '，'
            | '！'
            | '？'
            | '；'
            | '：'
            | '）'
            | '】'
            | '》'
            | '-'
            | '—'
    )
}

fn needs_space_between(previous: &str, next: &str) -> bool {
    let (Some(prev), Some(first)) = (previous.chars().next_back(), next.chars().next()) else {
        return false;
    };
    if prev.is_whitespace() || first.is_whitespace() {
        return false;
    }
    if is_opening_punct(prev) || is_attaching_punct(first) {
        return false;
    }
    !(is_cjk(prev) && is_cjk(first))
}

#[derive(Default)]
struct ActiveCue {
    start_ms: Option<u64>,
    end_ms: u64,
    text: String,
    chars: usize,
    cjk_chars: usize,
}

impl ActiveCue {
    fn is_cjk_dominant(&self) -> bool {
        self.cjk_chars * 3 >= self.chars.max(1)
    }

    fn should_end_before(&self, token: &TimedToken) -> bool {
        let Some(start_ms) = self.start_ms else {
            return false;
        };
        if token.start_ms.saturating_sub(self.end_ms) >= SILENCE_BOUNDARY_MS
            || token.end_ms.saturating_sub(start_ms) > MAX_CUE_DURATION_MS
        {
            return true;
        }
        let next_chars = token.text.trim().chars().count();
        let hard_limit = if self.is_cjk_dominant() {
            HARD_CJK_CHARS
        } else {
            HARD_LATIN_CHARS
        };
        self.chars > 0 && self.chars + next_chars + 1 > hard_limit
    }

    fn push(&mut self, token: TimedToken) {
        self.start_ms.get_or_insert(token.start_ms);
        self.end_ms = token.end_ms;
        if needs_space_between(&self.text, &token.text) {
            self.text.push(' ');
            self.chars += 1;
        }
        for ch in token.text.chars() {
            self.chars += 1;
            if is_cjk(ch) {
                self.cjk_chars += 1;
            }
        }
        self.text.push_str(&token.text);
    }

    fn ends_sentence(&self) -> bool {
        self.text
            .trim_end()
            .chars()
            .next_back()
            .is_some_and(|value| matches!(value, '。' | '！' | '？' | '!' | '?' | '.' | '…'))
    }

    fn should_end_after_clause(&self) -> bool {
        let soft_limit = if self.is_cjk_dominant() {
            SOFT_CJK_CHARS
        } else {
            SOFT_LATIN_CHARS
        };
        self.chars >= soft_limit
            && self
                .text
                .trim_end()
                .chars()
                .next_back()
                .is_some_and(|value| matches!(value, ',' | ';' | ':' | '，' | '；' | '：' | '、'))
    }

    fn finish(&mut self, output: &mut Vec<SubtitleCue>) {
        let Some(start_ms) = self.start_ms.take() else {
            return;
        };
        let text = std::mem::take(&mut self.text).trim().to_owned();
        if !text.is_empty() && self.end_ms > start_ms {
            output.push(SubtitleCue {
                start_ms,
                end_ms: self.end_ms,
                text,
            });
        }
        self.end_ms = 0;
        self.chars = 0;
        self.cjk_chars = 0;
    }
}
