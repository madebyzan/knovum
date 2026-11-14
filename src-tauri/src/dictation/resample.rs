pub(crate) const TARGET_SAMPLE_RATE: u32 = 16_000;
pub(crate) const TARGET_CHANNELS: u16 = 1;

/// Converts raw audio samples from any captured sample rate and channel layout into
/// 16kHz mono PCM samples required by the Whisper ASR engine.
pub(crate) fn to_16k_mono(samples: &[i16], channels: u16, in_rate: u32) -> Vec<i16> {
    if samples.is_empty() || channels == 0 || in_rate == 0 {
        return Vec::new();
    }

    // Step 1: Downmix channels to mono
    let mono: Vec<i16> = if channels == 1 {
        samples.to_vec()
    } else {
        let ch = usize::from(channels);
        samples
            .chunks_exact(ch)
            .map(|frame| {
                let sum: i32 = frame.iter().map(|&s| i32::from(s)).sum();
                (sum / ch as i32) as i16
            })
            .collect()
    };

    // Step 2: Resample to TARGET_SAMPLE_RATE (16,000 Hz)
    if in_rate == TARGET_SAMPLE_RATE {
        return mono;
    }

    let ratio = f64::from(in_rate) / f64::from(TARGET_SAMPLE_RATE);
    let output_len = ((mono.len() as f64) / ratio).round() as usize;
    if output_len == 0 {
        return Vec::new();
    }
    let mut output = Vec::with_capacity(output_len);

    if ratio >= 1.0 {
        // Downsampling: area-average over [start, end] to prevent aliasing
        for out_idx in 0..output_len {
            let start = out_idx as f64 * ratio;
            let end = ((out_idx + 1) as f64 * ratio).min(mono.len() as f64);
            let start_idx = start.floor() as usize;
            let end_idx = (end.ceil() as usize).min(mono.len());
            let mut sum = 0.0f64;
            let mut total_weight = 0.0f64;
            for (j, &sample) in mono.iter().enumerate().take(end_idx).skip(start_idx) {
                let left = (j as f64).max(start);
                let right = ((j + 1) as f64).min(end);
                let weight = (right - left).max(0.0);
                sum += f64::from(sample) * weight;
                total_weight += weight;
            }
            let val = if total_weight > 0.0 {
                sum / total_weight
            } else {
                0.0
            };
            output.push(val.round().clamp(-32_768.0, 32_767.0) as i16);
        }
    } else {
        // Upsampling: linear interpolation
        for out_idx in 0..output_len {
            let src = out_idx as f64 * ratio;
            let idx0 = src.floor() as usize;
            let frac = (src - idx0 as f64) as f32;
            let s0 = f32::from(mono[idx0.min(mono.len() - 1)]);
            let s1 = f32::from(mono[(idx0 + 1).min(mono.len() - 1)]);
            let interpolated = s0 + frac * (s1 - s0);
            output.push(interpolated.round().clamp(-32_768.0, 32_767.0) as i16);
        }
    }

    output
}

#[cfg(test)]
#[path = "tests/resample.rs"]
mod tests;
