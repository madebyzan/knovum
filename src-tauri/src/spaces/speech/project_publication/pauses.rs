use std::path::Path;

use tokio_util::sync::CancellationToken;

use super::encoding::NormalizedSource;

pub(super) fn insert(
    sources: &[NormalizedSource],
    sample_rate: u32,
    pause_ms: u32,
    directory: &Path,
    cancellation: &CancellationToken,
) -> Result<Vec<NormalizedSource>, String> {
    let samples = u64::from(sample_rate) * u64::from(pause_ms) / 1_000;
    let silence = directory.join("pause.wav");
    if samples > 0 && sources.len() > 1 {
        let mut writer = hound::WavWriter::create(
            &silence,
            hound::WavSpec {
                channels: 1,
                sample_rate,
                bits_per_sample: 32,
                sample_format: hound::SampleFormat::Float,
            },
        )
        .map_err(|error| error.to_string())?;
        for index in 0..samples {
            if index % 16_384 == 0 && cancellation.is_cancelled() {
                return Err("cancelled".into());
            }
            writer
                .write_sample(0.0_f32)
                .map_err(|error| error.to_string())?;
        }
        writer.finalize().map_err(|error| error.to_string())?;
    }
    let mut result = Vec::with_capacity(sources.len().saturating_mul(2));
    for (index, source) in sources.iter().enumerate() {
        if index > 0 && samples > 0 {
            result.push(NormalizedSource {
                path: silence.clone(),
                sample_count: samples,
            });
        }
        result.push(NormalizedSource {
            path: source.path.clone(),
            sample_count: source.sample_count,
        });
    }
    Ok(result)
}
