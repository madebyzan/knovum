use super::{MediaTranscript, SubtitleCue, TranscriptSegment};
use crate::{
    ai::contracts::speech_wire::SpeechTranscriptArtifact,
    media::subtitle_timing::{self, TimedToken},
};

pub(crate) fn from_speech(
    artifact: SpeechTranscriptArtifact,
    audio_duration_ms: u64,
) -> Result<MediaTranscript, String> {
    let language = artifact.language.trim();
    if language.is_empty() || language.len() > 64 {
        return Err("invalid_output: ASR language is invalid".into());
    }
    if artifact.duration_ms > audio_duration_ms.saturating_add(1_000) {
        return Err("invalid_output: ASR duration exceeds the source audio".into());
    }
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
        if segment.words.is_empty() {
            tokens.push(TimedToken {
                start_ms: segment.start_ms,
                end_ms: segment.end_ms,
                text: text.to_owned(),
            });
        } else {
            let mut previous_word_start = segment.start_ms;
            for word in segment.words {
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
                tokens.push(TimedToken {
                    start_ms: word.start_ms,
                    end_ms: word.end_ms,
                    text: word.text,
                });
            }
        }
        segments.push(TranscriptSegment {
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
    Ok(MediaTranscript {
        language: language.to_owned(),
        duration_ms: audio_duration_ms,
        text,
        segments,
        subtitle_cues,
    })
}
