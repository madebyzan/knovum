use crate::spaces::speech::{
    contracts::SpeechSegmentRenderOutput, project_config::AudiobookOutputConfig,
};
use crate::spaces::{
    db,
    speech::{audio_format::WavPrecision, project_config::ShortOutputConfig},
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
    match project_type {
        "short_dubbing" => {
            crate::spaces::speech::project_config::validate(project_type, output_config)?;
            let first = renders
                .first()
                .ok_or("Speech project has no rendered segments")?
                .sample_rate;
            let rate = if renders.iter().all(|render| render.sample_rate == first) {
                first
            } else {
                48_000
            };
            let config = serde_json::from_value::<ShortOutputConfig>(output_config.clone())
                .map_err(db::json_error)?;
            Ok(ProjectOutputProfile {
                format: "wav".into(),
                sample_rate: rate,
                wav_precision: config.wav_precision,
                audiobook: None,
            })
        }
        "audiobook" => {
            let config = crate::spaces::speech::project_config::parse_audiobook(output_config)?;
            Ok(ProjectOutputProfile {
                format: config.format.clone(),
                sample_rate: config.sample_rate,
                wav_precision: config.wav_precision,
                audiobook: Some(config),
            })
        }
        _ => Err("This speech project type has no complete production workflow".into()),
    }
}

pub(super) fn digest_render_order(
    renders: &[SpeechSegmentRenderOutput],
    output_config: &serde_json::Value,
) -> String {
    let mut digest = Sha256::new();
    for render in renders {
        digest.update(render.segment_id.as_bytes());
        digest.update([0]);
        digest.update(render.sha256.as_bytes());
        digest.update([0]);
    }
    digest.update(output_config.to_string().as_bytes());
    format!("{:x}", digest.finalize())
}
