use crate::spaces::speech::{
    contracts::SpeechSegmentRenderOutput, project_config::AudiobookOutputConfig,
};
use sha2::{Digest, Sha256};

pub(super) fn output_profile(
    project_type: &str,
    output_config: &serde_json::Value,
    renders: &[SpeechSegmentRenderOutput],
) -> Result<(String, u32, Option<AudiobookOutputConfig>), String> {
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
            Ok(("wav".into(), rate, None))
        }
        "audiobook" => {
            let config = crate::spaces::speech::project_config::parse_audiobook(output_config)?;
            Ok((config.format.clone(), config.sample_rate, Some(config)))
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
    if let Ok(encoded) = serde_json::to_vec(output_config) {
        digest.update(encoded);
    }
    format!("{:x}", digest.finalize())
}
