mod assembly;
mod contracts;
mod units;

#[cfg(test)]
pub(crate) mod tests;

pub(crate) use assembly::from_speech;
pub(crate) use contracts::{
    MediaTranscript, SpeakerTrack, SpeakerTurn, SubtitleCue, TranscriptAlignmentProvenance,
    TranscriptDiarizationProvenance, TranscriptProvenance, TranscriptSegment, TranscriptUnit,
};
