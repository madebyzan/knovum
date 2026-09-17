use super::{cues, MediaTranscript, SubtitleCue, TranscriptSegment};
use crate::media::whisper::WhisperOutput;

pub struct TranscriptAssembly {
    language: Option<String>,
    segments: Vec<TranscriptSegment>,
    subtitle_cues: Vec<SubtitleCue>,
}

impl TranscriptAssembly {
    pub fn new() -> Self {
        Self {
            language: None,
            segments: Vec::new(),
            subtitle_cues: Vec::new(),
        }
    }

    pub fn append(&mut self, part: WhisperOutput, duration_ms: u64) -> Vec<TranscriptSegment> {
        self.capture_language(&part.language);
        let before = self.segments.len();
        self.subtitle_cues
            .extend(part.subtitle_cues.into_iter().map(|cue| SubtitleCue {
                start_ms: cue.start_ms,
                end_ms: cue.end_ms,
                text: cue.text,
            }));
        for segment in part.segments {
            self.push_segment(segment.start, segment.end, segment.text, duration_ms);
        }
        self.segments[before..].to_vec()
    }

    pub fn language(&self) -> &str {
        self.language.as_deref().unwrap_or("unknown")
    }

    pub fn finish(
        mut self,
        waveform: Vec<f32>,
        duration_ms: u64,
    ) -> Result<MediaTranscript, String> {
        self.segments.sort_by_key(|segment| segment.start_ms);
        self.subtitle_cues = cues::sanitize(self.subtitle_cues, duration_ms);
        let text = self
            .segments
            .iter()
            .map(|segment| segment.text.as_str())
            .collect::<Vec<_>>()
            .join(" ");
        Ok(MediaTranscript {
            language: self.language.unwrap_or_else(|| "unknown".to_owned()),
            duration_ms,
            text,
            segments: self.segments,
            subtitle_cues: self.subtitle_cues,
            waveform,
        })
    }

    fn capture_language(&mut self, language: &str) {
        if self.language.is_none() && !language.trim().is_empty() {
            self.language = Some(language.trim().to_owned());
        }
    }

    fn push_segment(&mut self, start: f64, end: f64, text: String, duration_ms: u64) {
        let text = text.trim().to_owned();
        let start_ms = seconds_to_ms(start);
        let end_ms = seconds_to_ms(end).min(duration_ms);
        if text.is_empty()
            || end_ms <= start_ms
            || duplicate_boundary(&self.segments, start_ms, end_ms, &text)
        {
            return;
        }
        self.segments.push(TranscriptSegment {
            start_ms,
            end_ms,
            text,
        });
    }
}

fn seconds_to_ms(value: f64) -> u64 {
    if value.is_finite() && value > 0.0 {
        (value * 1_000.0).round() as u64
    } else {
        0
    }
}

fn duplicate_boundary(
    segments: &[TranscriptSegment],
    start_ms: u64,
    end_ms: u64,
    text: &str,
) -> bool {
    segments.last().is_some_and(|previous| {
        previous.text == text
            && start_ms <= previous.end_ms.saturating_add(800)
            && end_ms.saturating_add(800) >= previous.start_ms
    })
}
