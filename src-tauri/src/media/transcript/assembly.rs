use super::{
    contracts::NormalizedTranscriptSegment, units, MediaTranscript, SubtitleCue,
    TranscriptProvenance,
};
use crate::{
    ai::contracts::speech_wire::SpeechTranscriptArtifact,
    media::subtitle_timing::{self, TimedToken},
};

/// Builds word-accurate subtitle cues from normalized segments while preserving the original
/// inter-word spacing and punctuation from each segment's text.
pub(crate) fn cues_from_segments(segments: &[NormalizedTranscriptSegment]) -> Vec<SubtitleCue> {
    let mut tokens = Vec::new();
    for segment in segments {
        let has_timed_units = !segment.units.is_empty()
            && segment
                .units
                .iter()
                .all(|unit| unit.start_ms.is_some() && unit.end_ms.is_some());
        if has_timed_units {
            let chars: Vec<char> = segment.text.chars().collect();
            let mut cursor = 0usize;
            for (index, unit) in segment.units.iter().enumerate() {
                let (Some(start_ms), Some(end_ms)) = (unit.start_ms, unit.end_ms) else {
                    continue;
                };
                let start_char = (unit.start_character as usize).min(chars.len());
                let end_char = (unit.end_character as usize).min(chars.len());
                let text = if start_char <= end_char && end_char > start_char {
                    let slice_start = if index == 0 {
                        start_char
                    } else {
                        cursor.min(start_char)
                    };
                    let slice_end = if index + 1 == segment.units.len() {
                        chars.len()
                    } else {
                        end_char
                    };
                    cursor = slice_end;
                    chars[slice_start..slice_end].iter().collect()
                } else {
                    unit.text.clone()
                };
                tokens.push(TimedToken {
                    start_ms,
                    end_ms,
                    text,
                });
            }
        } else {
            tokens.push(TimedToken {
                start_ms: segment.start_ms,
                end_ms: segment.end_ms,
                text: segment.text.clone(),
            });
        }
    }
    subtitle_timing::cues(tokens)
        .into_iter()
        .map(|cue| SubtitleCue {
            start_ms: cue.start_ms,
            end_ms: cue.end_ms,
            text: cue.text,
        })
        .collect()
}

pub(crate) fn from_speech(
    artifact: SpeechTranscriptArtifact,
    audio_duration_ms: u64,
    source_revision: &str,
    mut provenance: TranscriptProvenance,
) -> Result<MediaTranscript, String> {
    let language = artifact.language.trim();
    if language.is_empty() || language.len() > 64 {
        return Err("invalid_output: ASR language is invalid".into());
    }
    if artifact.duration_ms > audio_duration_ms.saturating_add(1_000)
        || artifact.speech_duration_ms > artifact.duration_ms
    {
        return Err("invalid_output: ASR duration exceeds the source audio".into());
    }
    let transcript_id = units::stable_id(source_revision, &provenance.model_spec_id);
    let mut has_engine_words = false;
    let mut segments = Vec::with_capacity(artifact.segments.len());
    let mut previous_start = 0;
    for (index, segment) in artifact.segments.into_iter().enumerate() {
        let text = segment.text.trim();
        if text.is_empty()
            || segment.end_ms <= segment.start_ms
            || segment.end_ms > audio_duration_ms
            || (index > 0 && segment.start_ms < previous_start)
        {
            return Err("invalid_output: ASR segment is invalid or out of order".into());
        }
        previous_start = segment.start_ms;
        let segment_id = units::stable_id(
            &transcript_id,
            &format!("{index}:{}:{}", segment.start_ms, segment.end_ms),
        );
        let normalized_units = units::from_words(&segment_id, text, &segment.words)?;
        has_engine_words |= normalized_units.iter().any(|unit| unit.start_ms.is_some());
        let mut previous_word_start = segment.start_ms;
        for word in &segment.words {
            if word.text.trim().is_empty()
                || word.end_ms <= word.start_ms
                || word.start_ms < segment.start_ms
                || word.start_ms < previous_word_start
                || word.end_ms > segment.end_ms
                || word
                    .probability
                    .is_some_and(|value| !value.is_finite() || !(0.0..=1.0).contains(&value))
            {
                return Err("invalid_output: ASR word timing is invalid".into());
            }
            previous_word_start = word.start_ms;
        }
        segments.push(NormalizedTranscriptSegment {
            id: segment_id,
            revision: 1,
            start_sample: segment
                .start_ms
                .checked_mul(16)
                .ok_or("Transcript sample offset overflow")?,
            end_sample: segment
                .end_ms
                .checked_mul(16)
                .ok_or("Transcript sample offset overflow")?,
            speaker_id: None,
            units: normalized_units,
            start_ms: segment.start_ms,
            end_ms: segment.end_ms,
            text: text.to_owned(),
        });
    }
    let subtitle_cues = cues_from_segments(&segments);
    let text = segments
        .iter()
        .map(|segment| segment.text.as_str())
        .collect::<Vec<_>>()
        .join(" ");
    provenance.timing = if has_engine_words {
        "engine_words"
    } else {
        "segment_only"
    }
    .into();
    Ok(MediaTranscript {
        id: transcript_id,
        source_revision: source_revision.into(),
        revision: 1,
        provenance,
        speaker_tracks: Vec::new(),
        alignment: None,
        diarization: None,
        language: language.to_owned(),
        duration_ms: audio_duration_ms,
        text,
        segments,
        subtitle_cues,
    })
}
