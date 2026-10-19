use crate::spaces::speech::{audio_format::WavPrecision, project_config::ProjectOutputConfig};
use crate::spaces::speech::{
    contracts::SpeechSegmentRenderOutput, project_config::AudiobookOutputConfig,
};
use sha2::{Digest, Sha256};

#[derive(Clone)]
pub(super) struct ProjectOutputProfile {
    pub format: String,
    pub sample_rate: u32,
    pub wav_precision: WavPrecision,
    pub audiobook: Option<AudiobookOutputConfig>,
}

pub(super) fn output_profile(
    project_type: &str,
    output_config: &serde_json::Value,
    renders: &[SpeechSegmentRenderOutput],
) -> Result<ProjectOutputProfile, String> {
    match crate::spaces::speech::project_config::parse(project_type, output_config)? {
        ProjectOutputConfig::ShortDubbing(config) => {
            let first = renders
                .first()
                .ok_or("Speech project has no rendered segments")?
                .sample_rate;
            let rate = if renders.iter().all(|render| render.sample_rate == first) {
                first
            } else {
                48_000
            };
            Ok(ProjectOutputProfile {
                format: "wav".into(),
                sample_rate: rate,
                wav_precision: config.wav_precision,
                audiobook: None,
            })
        }
        ProjectOutputConfig::Audiobook(config) => Ok(ProjectOutputProfile {
            format: config.format.clone(),
            sample_rate: config.sample_rate,
            wav_precision: config.wav_precision,
            audiobook: Some(config),
        }),
        _ => Err("This speech project type has no complete production workflow".into()),
    }
}

/// Identifies how a project's audio is assembled, so a published output is only reused while the
/// rules that produced it still hold. It changes whenever line trimming or pausing changes.
const ASSEMBLY_RULES: &str = "speech-assembly-v2-trimmed-lines";

pub(super) fn digest_render_order(
    renders: &[SpeechSegmentRenderOutput],
    output_config: &serde_json::Value,
) -> String {
    let mut digest = Sha256::new();
    digest.update(ASSEMBLY_RULES.as_bytes());
    for render in renders {
        digest.update(render.segment_id.as_bytes());
        digest.update([0]);
        digest.update(render.sha256.as_bytes());
        digest.update([0]);
    }
    digest.update(output_config.to_string().as_bytes());
    hex::encode(digest.finalize())
}
