use knovum_inference_contracts::speech_wire::SpeechWordTiming;

pub(super) fn append(
    words: &mut Vec<SpeechWordTiming>,
    piece: &str,
    start_ms: u64,
    end_ms: u64,
    probability: f32,
    language: &str,
) {
    if matches!(language, "zh" | "ja" | "th" | "lo" | "my" | "yue")
        || piece.starts_with(char::is_whitespace)
        || words.is_empty()
    {
        words.push(SpeechWordTiming {
            text: piece.into(),
            start_ms,
            end_ms,
            probability: Some(probability),
        });
    } else if let Some(word) = words.last_mut() {
        word.text.push_str(piece);
        word.end_ms = end_ms;
        word.probability = Some(probability);
    }
}

pub(super) fn finish(words: &mut Vec<SpeechWordTiming>, start_ms: u64, end_ms: u64) {
    for word in words.iter_mut() {
        word.text = word.text.trim().to_owned();
        word.start_ms = word.start_ms.max(start_ms);
        word.end_ms = word.end_ms.min(end_ms);
    }
    words.retain(|word| !word.text.is_empty() && word.end_ms > word.start_ms);
}

#[cfg(test)]
#[path = "tests/whisper_words.rs"]
mod tests;
