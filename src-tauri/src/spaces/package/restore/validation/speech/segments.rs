use super::{reference, required, SpeechIndex};
use crate::spaces::package::restore::contracts::RestorePackageRequest;
use std::collections::{HashMap, HashSet};

pub(super) fn validate(
    request: &RestorePackageRequest,
    index: &SpeechIndex<'_>,
    lexicons: &HashMap<&str, Vec<(String, String)>>,
) -> Result<(), String> {
    let SpeechIndex {
        revisions,
        project_revisions,
        voice_languages,
        voice_deliveries,
        ..
    } = index;
    let mut ordinals = HashSet::new();
    for row in &request.speech_segments {
        reference(
            project_revisions,
            &row.project_revision_id,
            "speech segment revision",
        )?;
        reference(revisions, &row.voice_revision_id, "speech segment voice")?;
        if row.ordinal < 0 || !ordinals.insert((row.project_revision_id.as_str(), row.ordinal)) {
            return Err(
                "Package speech segment ordinals must be nonnegative and unique per revision"
                    .into(),
            );
        }
        required(&row.text, usize::MAX, "Speech segment text")?;
        required(&row.normalized_text, usize::MAX, "Speech normalized text")?;
        let pronunciations = lexicons
            .get(row.project_revision_id.as_str())
            .ok_or("Package speech segment revision is unavailable")?;
        if row.normalized_text
            != crate::spaces::speech::text::apply_lexicon(&row.text, pronunciations)
        {
            return Err(
                "Package speech segment normalized text differs from its source and lexicon".into(),
            );
        }
        required(&row.language, usize::MAX, "Speech segment language")?;
        if voice_languages.get(row.voice_revision_id.as_str()) != Some(&row.language.as_str()) {
            return Err("Package speech segment language differs from its voice revision".into());
        }
        let supported = voice_deliveries
            .get(row.voice_revision_id.as_str())
            .ok_or("Package speech voice revision is unavailable")?;
        let delivery = crate::spaces::speech::delivery_instruction::validate(
            row.delivery_instruction.as_deref(),
            *supported,
        )?;
        if delivery != row.delivery_instruction.as_deref() {
            return Err("Package delivery instruction is not canonical".into());
        }
        if !matches!(
            row.state.as_str(),
            "pending" | "ready" | "rendered" | "needs_edit" | "failed"
        ) {
            return Err("Package speech segment state is not stable".into());
        }
        match (row.target_start_ms, row.target_end_ms) {
            (None, None) => {}
            (Some(start), Some(end)) if start >= 0 && end > start => {}
            _ => return Err("Package speech segment timing is invalid".into()),
        }
        match (
            row.source_segment_id.as_deref(),
            row.source_segment_revision,
            row.source_text.as_deref(),
        ) {
            (None, None, None) => {}
            (Some(id), Some(revision), Some(text))
                if !id.trim().is_empty()
                    && id.encode_utf16().count() <= 240
                    && revision > 0
                    && !text.trim().is_empty()
                    && text.chars().count() <= 16_000 => {}
            _ => return Err("Package speech segment source identity is invalid".into()),
        }
        if row.created_at < 0 || row.updated_at < row.created_at {
            return Err("Package speech segment timestamps are invalid".into());
        }
    }

    Ok(())
}
