use sha2::{Digest, Sha256};

use crate::ai::contracts::speech_wire::SpeechWordTiming;

use super::types::TranscriptUnit;

pub(super) fn stable_id(scope: &str, key: &str) -> String {
    let mut hash = Sha256::new();
    hash.update(scope.as_bytes());
    hash.update([0]);
    hash.update(key.as_bytes());
    format!("{:x}", hash.finalize())
}

pub(super) fn from_words(
    segment_id: &str,
    text: &str,
    words: &[SpeechWordTiming],
) -> Result<Vec<TranscriptUnit>, String> {
    if words.is_empty() {
        return unaligned(segment_id, text, "word_timing_unavailable");
    }
    let mut units = Vec::with_capacity(words.len());
    let mut byte_offset = 0;
    for (index, word) in words.iter().enumerate() {
        let token = word.text.trim();
        let Some(relative) = text[byte_offset..].find(token) else {
            return unaligned(segment_id, text, "asr_text_mismatch");
        };
        let start = byte_offset + relative;
        let end = start + token.len();
        units.push(TranscriptUnit {
            id: stable_id(segment_id, &index.to_string()),
            text: token.into(),
            start_character: character_offset(&text[..start])?,
            end_character: character_offset(&text[..end])?,
            start_ms: Some(word.start_ms),
            end_ms: Some(word.end_ms),
            score: word.probability,
            unaligned_reason: None,
            unit_kind: "word".into(),
        });
        byte_offset = end;
    }
    Ok(units)
}

fn character_offset(text: &str) -> Result<u32, String> {
    u32::try_from(text.chars().count())
        .map_err(|_| "Transcript character offset exceeds its range".into())
}

fn unaligned(segment_id: &str, text: &str, reason: &str) -> Result<Vec<TranscriptUnit>, String> {
    Ok(vec![TranscriptUnit {
        id: stable_id(segment_id, "unaligned"),
        text: text.into(),
        start_character: 0,
        end_character: character_offset(text)?,
        start_ms: None,
        end_ms: None,
        score: None,
        unaligned_reason: Some(reason.into()),
        unit_kind: "segment".into(),
    }])
}
