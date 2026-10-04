mod assembly;
mod types;
mod units;

#[cfg(test)]
pub(crate) mod tests;

pub(crate) use assembly::from_speech;
pub(crate) use types::{
    MediaTranscript, SpeakerTrack, SpeakerTurn, SubtitleCue, TranscriptAlignmentProvenance,
    TranscriptDiarizationProvenance, TranscriptProvenance, TranscriptSegment, TranscriptUnit,
};
