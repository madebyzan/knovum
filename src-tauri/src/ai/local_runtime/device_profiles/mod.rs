use crate::ai::contracts::runtime::{RuntimeCapability, RuntimeDeviceProfile};

mod memory;

const GIB: u64 = 1024 * 1024 * 1024;

#[derive(Debug)]
pub(super) struct MemoryRequirement {
    pub load_peak_bytes: u64,
    pub inference_peak_bytes: u64,
    pub profile_name: &'static str,
}

impl MemoryRequirement {
    pub fn peak_bytes(&self) -> u64 {
        self.load_peak_bytes.max(self.inference_peak_bytes)
    }
}

#[derive(Clone, Copy)]
pub(super) struct MemorySnapshot {
    pub total_bytes: u64,
    pub available_bytes: u64,
}

/// Conservative process peak reservations tied to exact frozen execution profiles.
/// Release qualification must validate them and raise any budget below a measured peak.
pub(super) fn memory_requirement(
    capability: RuntimeCapability,
    device_profile: &RuntimeDeviceProfile,
) -> Result<MemoryRequirement, String> {
    let id = device_profile.execution_profile_id.as_str();
    let device = device_profile.device.as_str();
    let (load_peak_gib, inference_peak_gib, profile_name) = match (capability, id, device) {
        (RuntimeCapability::Embedding, _, "cpu")
            if valid_threaded_profile(device_profile, "cpu-f32-threads") =>
        {
            (3, 4, "Qwen3 embedding CPU")
        }
        (RuntimeCapability::ImageSegmentation, _, "cpu")
            if valid_threaded_profile(device_profile, "cpu-f32-threads") =>
        {
            (1, 1, "U2Net image segmentation CPU")
        }
        (RuntimeCapability::SpeechTranscription, "mlx-whisper-small-mps-v1", "mps") => {
            (2, 3, "MLX Whisper small MPS")
        }
        (RuntimeCapability::SpeechTranscription, "faster-whisper-small-cpu-int8-v1", "cpu") => {
            (2, 3, "faster-whisper small CPU INT8")
        }
        (RuntimeCapability::SpeechAlignment, "qwen3-forced-aligner-0.6b-cpu-v1", "cpu") => {
            (5, 6, "Qwen3 forced aligner CPU")
        }
        (RuntimeCapability::SpeechDiarization, "community1-knovum-cpu-v1", "cpu") => {
            (2, 4, "Pyannote Community-1 CPU")
        }
        (RuntimeCapability::SpeechSeparation, "spleeter-2stems-fp16-cpu-v1", "cpu") => {
            (2, 8, "Spleeter 2-stem CPU FP16")
        }
        (RuntimeCapability::SpeechSynthesis, "voxcpm2-mps-f32-v1", "mps") => {
            (20, 24, "VoxCPM2 MPS F32")
        }
        (RuntimeCapability::SpeechSynthesis, "voxcpm2-cpu-bf16-v1", "cpu") => {
            (14, 16, "VoxCPM2 CPU BF16")
        }
        (RuntimeCapability::SpeechSynthesis, _, "cpu")
            if valid_threaded_profile(device_profile, "cpu-int8-threads") =>
        {
            (1, 2, "Kokoro CPU INT8")
        }
        _ => return Err("runtime_mismatch: execution profile has no memory manifest".into()),
    };
    Ok(MemoryRequirement {
        load_peak_bytes: load_peak_gib * GIB,
        inference_peak_bytes: inference_peak_gib * GIB,
        profile_name,
    })
}

fn valid_threaded_profile(profile: &RuntimeDeviceProfile, prefix: &str) -> bool {
    profile
        .execution_profile_id
        .strip_prefix(prefix)
        .and_then(|value| value.strip_suffix("-v1"))
        .and_then(|value| value.parse::<u8>().ok())
        .is_some_and(|threads| (1..=4).contains(&threads) && threads == profile.cpu_threads)
}

pub(super) fn application_memory_budget(total_bytes: u64) -> u64 {
    total_bytes
        .saturating_sub(4 * GIB)
        .min(total_bytes.saturating_mul(2) / 3)
}

pub(super) fn memory_snapshot() -> Result<MemorySnapshot, String> {
    let snapshot = memory::platform_memory_snapshot()?;
    if snapshot.total_bytes == 0 || snapshot.available_bytes == 0 {
        return Err("resource_exhausted: Platform memory statistics are invalid".into());
    }
    Ok(snapshot)
}

/// Advanced synthesis has a separate physical-memory floor from baseline capabilities.
pub(crate) fn voxcpm2() -> Result<RuntimeDeviceProfile, String> {
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    let (device, precision, minimum_gib) = ("mps", "f32", 48);
    #[cfg(all(
        any(target_os = "macos", target_os = "windows"),
        target_arch = "x86_64"
    ))]
    let (device, precision, minimum_gib) = ("cpu", "bf16", 32);
    #[cfg(not(knovum_supported_desktop))]
    return Err(
        "unsupported_device: Voice cloning and design require macOS arm64/x64 or Windows x64"
            .into(),
    );
    #[cfg(knovum_supported_desktop)]
    {
        let memory = memory::physical_memory()?;
        if memory < minimum_gib * GIB {
            return Err(format!("resource_exhausted: Voice cloning and design require at least {minimum_gib} GiB physical memory for the {device}/{precision} profile"));
        }
        Ok(RuntimeDeviceProfile {
            device: device.into(),
            execution_profile_id: format!("voxcpm2-{device}-{precision}-v1"),
            cpu_threads: crate::ai::local_runtime::compute_budget::ComputeBudget::detect()?
                .model_threads,
        })
    }
}

pub(crate) fn speech_separation() -> Result<RuntimeDeviceProfile, String> {
    #[cfg(not(knovum_supported_desktop))]
    return Err(
        "unsupported_device: Source separation requires macOS arm64/x64 or Windows x64".into(),
    );
    #[cfg(knovum_supported_desktop)]
    Ok(RuntimeDeviceProfile {
        device: "cpu".into(),
        execution_profile_id: "spleeter-2stems-fp16-cpu-v1".into(),
        cpu_threads: crate::ai::local_runtime::compute_budget::ComputeBudget::detect()?
            .model_threads,
    })
}

#[cfg(test)]
mod tests;
