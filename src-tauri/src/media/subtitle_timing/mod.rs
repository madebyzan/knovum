const SILENCE_BOUNDARY_MS: u64 = 650;
const MAX_CUE_DURATION_MS: u64 = 5_000;

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
        if active.ends_sentence() {
            active.finish(&mut output);
        }
    }
    active.finish(&mut output);
    output
}

fn valid(token: &TimedToken) -> bool {
    token.end_ms > token.start_ms && !token.text.trim().is_empty()
}

#[derive(Default)]
struct ActiveCue {
    start_ms: Option<u64>,
    end_ms: u64,
    text: String,
}

impl ActiveCue {
    fn should_end_before(&self, token: &TimedToken) -> bool {
        let Some(start_ms) = self.start_ms else {
            return false;
        };
        token.start_ms.saturating_sub(self.end_ms) >= SILENCE_BOUNDARY_MS
            || token.end_ms.saturating_sub(start_ms) > MAX_CUE_DURATION_MS
    }

    fn push(&mut self, token: TimedToken) {
        self.start_ms.get_or_insert(token.start_ms);
        self.end_ms = token.end_ms;
        self.text.push_str(&token.text);
    }

    fn ends_sentence(&self) -> bool {
        self.text
            .trim_end()
            .chars()
            .next_back()
            .is_some_and(|value| matches!(value, '。' | '！' | '？' | '!' | '?' | '.' | '…'))
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
    }
}
