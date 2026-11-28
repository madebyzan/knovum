mod assembly;
mod contracts;
mod units;

#[cfg(test)]
pub(crate) mod tests;

pub(crate) use assembly::{cues_from_segments, from_speech};
pub(crate) use contracts::{
    MediaTranscript, NormalizedTranscriptSegment, SpeakerTrack, SpeakerTurn, SubtitleCue,
    TranscriptAlignmentProvenance, TranscriptDiarizationProvenance, TranscriptProvenance,
    TranscriptSegment, TranscriptUnit,
};
