use super::{require_bounded_text, require_safe_timestamp, MAX_SPEAKER_TRACKS, MAX_SPEAKER_TURNS};
use crate::media::transcript::{MediaTranscript, SpeakerTrack, TranscriptProvenance};
use std::collections::HashSet;

pub(super) fn validate_speaker_tracks(
    transcript: &MediaTranscript,
) -> Result<HashSet<&str>, String> {
    if transcript.speaker_tracks.len() > MAX_SPEAKER_TRACKS {
        return Err("Transcript speaker tracks count is invalid".into());
    }
    if transcript.diarization.is_none()
        && (!transcript.speaker_tracks.is_empty()
            || transcript
                .segments
                .iter()
                .any(|segment| segment.speaker_id.is_some()))
    {
        return Err("Transcript speaker data requires diarization provenance".into());
    }
    let mut ids = HashSet::with_capacity(transcript.speaker_tracks.len());
    for track in &transcript.speaker_tracks {
        validate_speaker_track(track, transcript.duration_ms)?;
        if !ids.insert(track.speaker_id.as_str()) {
            return Err("Transcript speaker track ids must be unique".into());
        }
    }
    Ok(ids)
}

fn validate_speaker_track(track: &SpeakerTrack, duration: u64) -> Result<(), String> {
    require_bounded_text(&track.speaker_id, 240, "Transcript speaker id")?;
    if track.turns.is_empty() || track.turns.len() > MAX_SPEAKER_TURNS {
        return Err("Transcript speaker turns count is invalid".into());
    }
    let mut previous_end = None;
    for turn in &track.turns {
        require_safe_timestamp(turn.start_ms, "Transcript speaker turn start")?;
        require_safe_timestamp(turn.end_ms, "Transcript speaker turn end")?;
        if turn.end_ms <= turn.start_ms
            || turn.end_ms > duration
            || previous_end.is_some_and(|previous| turn.start_ms < previous)
        {
            return Err(
                "Transcript speaker turns must be ordered, non-overlapping, and bounded".into(),
            );
        }
        previous_end = Some(turn.end_ms);
    }
    Ok(())
}

pub(super) fn validate_provenance(provenance: &TranscriptProvenance) -> Result<(), String> {
    require_bounded_text(&provenance.engine_id, 240, "Transcript provenance engineId")?;
    require_bounded_text(
        &provenance.model_spec_id,
        240,
        "Transcript provenance modelSpecId",
    )?;
    require_bounded_text(
        &provenance.model_revision,
        240,
        "Transcript provenance modelRevision",
    )?;
    if !matches!(provenance.timing.as_str(), "native_words" | "segment_only") {
        return Err("Transcript provenance timing is invalid".into());
    }
    Ok(())
}

pub(super) fn validate_optional_provenance(transcript: &MediaTranscript) -> Result<(), String> {
    if let Some(alignment) = &transcript.alignment {
        require_bounded_text(&alignment.engine_id, 240, "Alignment engineId")?;
        require_bounded_text(&alignment.model_spec_id, 240, "Alignment modelSpecId")?;
        require_bounded_text(&alignment.model_revision, 240, "Alignment modelRevision")?;
        require_safe_timestamp(alignment.aligned_at, "Alignment timestamp")?;
    }
    if let Some(diarization) = &transcript.diarization {
        require_bounded_text(&diarization.engine_id, 240, "Diarization engineId")?;
        require_bounded_text(&diarization.model_spec_id, 240, "Diarization modelSpecId")?;
        require_bounded_text(
            &diarization.model_revision,
            240,
            "Diarization modelRevision",
        )?;
        require_safe_timestamp(diarization.diarized_at, "Diarization timestamp")?;
    }
    Ok(())
}
