use super::{
    encoding::{
        encode, normalize_sources, write_chapter_metadata, write_concat_file, EncodeRequest,
        LoudnessMeasurement,
    },
    output_spec::ProjectOutputProfile,
    pauses,
};
use crate::spaces::speech::contracts::SpeechSegmentRenderOutput;
use std::path::PathBuf;
use tokio_util::sync::CancellationToken;

pub(super) struct AssemblyInput {
    pub ffmpeg: PathBuf,
    pub sources: Vec<PathBuf>,
    pub renders: Vec<SpeechSegmentRenderOutput>,
    pub work_directory: PathBuf,
    pub output: PathBuf,
    pub profile: ProjectOutputProfile,
    pub chapter_keys: Vec<Option<String>>,
    pub measurement: Option<LoudnessMeasurement>,
}

/// What separates the lines of a plain read, matching the pause an audiobook puts between
/// paragraphs by default, so a short read is heard as sentences rather than one long one.
const SENTENCE_PAUSE_MS: u32 = 250;

pub(super) fn assemble(
    input: &AssemblyInput,
    cancellation: &CancellationToken,
) -> Result<u64, String> {
    let profile = &input.profile;
    let mut normalized = normalize_sources(
        &input.ffmpeg,
        &input.sources,
        &input.renders,
        profile.sample_rate,
        &input.work_directory,
        cancellation,
    )?;
    // Lines carry only their speech, so the read needs its own separation: an audiobook separates
    // the chapters it is assembled from, and a plain read separates its lines by a sentence pause.
    normalized = pauses::insert(
        &normalized,
        profile.sample_rate,
        profile
            .audiobook
            .as_ref()
            .map_or(SENTENCE_PAUSE_MS, |config| config.chapter_pause_ms),
        &input.work_directory,
        cancellation,
    )?;
    let sample_count = normalized.iter().try_fold(0_u64, |total, source| {
        total
            .checked_add(source.sample_count)
            .ok_or_else(|| "Speech project sample count overflow".to_owned())
    })?;
    let concat_path = write_concat_file(&input.work_directory, &normalized)?;
    let metadata_keys = chapter_metadata_keys(
        &input.chapter_keys,
        profile
            .audiobook
            .as_ref()
            .is_some_and(|config| config.chapter_pause_ms > 0),
    );
    let metadata_path = profile
        .audiobook
        .as_ref()
        .map(|config| {
            write_chapter_metadata(
                &input.work_directory,
                config,
                &metadata_keys,
                &normalized,
                profile.sample_rate,
            )
        })
        .transpose()?;
    encode(
        EncodeRequest {
            ffmpeg: &input.ffmpeg,
            concat_path: &concat_path,
            output: &input.output,
            format: &profile.format,
            sample_rate: profile.sample_rate,
            wav_precision: profile.wav_precision,
            audiobook: profile.audiobook.as_ref(),
            measurement: input.measurement.as_ref(),
            metadata: metadata_path.as_deref(),
        },
        cancellation,
    )?;
    Ok(sample_count)
}

pub(super) fn chapter_metadata_keys(
    keys: &[Option<String>],
    with_pauses: bool,
) -> Vec<Option<String>> {
    if !with_pauses {
        return keys.to_vec();
    }
    keys.iter()
        .enumerate()
        .flat_map(|(index, key)| {
            let mut entries = Vec::with_capacity(2);
            if index > 0 {
                entries.push(keys[index - 1].clone());
            }
            entries.push(key.clone());
            entries
        })
        .collect()
}
