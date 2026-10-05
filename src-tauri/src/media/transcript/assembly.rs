use super::{
    contracts::NormalizedTranscriptSegment, units, MediaTranscript, SubtitleCue,
    TranscriptProvenance,
};
use crate::{
    ai::contracts::speech_wire::SpeechTranscriptArtifact,
    media::subtitle_timing::{self, TimedToken},
};

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
    let mut tokens = Vec::new();
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
        if normalized_units
            .iter()
            .all(|unit| unit.start_ms.is_some() && unit.end_ms.is_some())
        {
            for unit in &normalized_units {
                tokens.push(TimedToken {
                    start_ms: unit
                        .start_ms
                        .ok_or("Aligned transcript unit has no start")?,
                    end_ms: unit.end_ms.ok_or("Aligned transcript unit has no end")?,
                    text: unit.text.clone(),
                });
            }
        } else {
            tokens.push(TimedToken {
                start_ms: segment.start_ms,
                end_ms: segment.end_ms,
                text: text.into(),
            });
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
    let subtitle_cues = subtitle_timing::cues(tokens)
        .into_iter()
        .map(|cue| SubtitleCue {
            start_ms: cue.start_ms,
            end_ms: cue.end_ms,
            text: cue.text,
        })
        .collect();
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
