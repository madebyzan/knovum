//! Canonical execution profile identities for the bundled inference runtime.
//!
//! An execution profile identity is the handshake between the host and an inference worker over one
//! frozen numeric setup: the host reserves memory for that exact profile and the worker refuses to
//! initialize anything else. Both sides derive every identity here, so a revision can never leave
//! one side producing what the other rejects.

use crate::runtime::RuntimeCapability;

/// Revision of the frozen bundled-runtime execution profiles. Bump it whenever the numerics behind
/// a profile change, so a stale host and worker refuse to pair instead of exchanging incomparable
/// results.
const REVISION: &str = "bundled-v2";

/// Whisper transcription variants as model and precision pairs, in ascending capability order.
pub const WHISPER_VARIANTS: [(&str, &str); 3] =
    [("tiny", "q8_0"), ("small", "q8_0"), ("large-v3", "f16")];

/// Identity of a profile whose only variable is its admitted CPU thread count.
pub fn threaded(prefix: &str, threads: u8) -> String {
    format!("{prefix}-threads{threads}-{REVISION}")
}

/// Qwen3 text embedding on the host-selected device.
pub fn embedding(device: &str, threads: u8) -> String {
    threaded(&format!("{device}-f32"), threads)
}

/// Whisper transcription for one variant of [`WHISPER_VARIANTS`].
pub fn whisper(model: &str, device: &str, precision: &str) -> String {
    format!("whisper-{model}-{device}-{precision}-{REVISION}")
}

/// ONNX image processing, which runs on CPU threads only.
pub fn image(capability: RuntimeCapability, threads: u8) -> Option<String> {
    let prefix = match capability {
        RuntimeCapability::ImageSegmentation => "cpu-f32",
        RuntimeCapability::ImageUpscaling => "image-upscaling-cpu",
        RuntimeCapability::ImageInpainting => "image-inpainting-cpu",
        RuntimeCapability::ImageSelection => "image-selection-cpu",
        RuntimeCapability::ImageDepth => "image-depth-cpu",
        _ => return None,
    };
    Some(threaded(prefix, threads))
}

/// MOSS-TTS-Nano synthesis, which runs on CPU threads only.
pub fn moss_nano(threads: u8) -> String {
    threaded("moss-nano-cpu-f32", threads)
}

/// Qwen3 forced alignment on a probed GGUF backend.
pub fn alignment(model_spec_id: &str, device: &str) -> Option<String> {
    let precision = match model_spec_id {
        "qwen3-forced-aligner-0.6b-f16-bundled-v2" => "f16",
        "qwen3-forced-aligner-0.6b-q8_0-v1" => "q8_0",
        _ => return None,
    };
    matches!(device, "cpu" | "mps" | "vulkan")
        .then(|| format!("qwen3-forced-aligner-0.6b-{precision}-{device}-{REVISION}"))
}

#[cfg(test)]
#[path = "tests/execution_profile.rs"]
mod tests;
