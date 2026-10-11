use super::ffi;
use std::path::Path;

pub(crate) fn devices(manifest: &Path, digest: &str, reserve: u64) -> Result<String, String> {
    crate::initialization::initialize_ort(manifest, digest)?;
    let path = ffi::library_path(manifest, "knovum_voice")?;
    let api = unsafe { ffi::voice::VoiceApi::from_library(ffi::load(&path)?) }
        .map_err(|error| format!("runtime_mismatch: {error}"))?;
    if unsafe { api.knovum_voice_abi() } != 3 {
        return Err("runtime_mismatch: voice C ABI differs".into());
    }
    let flags = unsafe { api.knovum_audio_devices(reserve) };
    if flags < 0 {
        return Err("unsupported_device: voice device enumeration failed".into());
    }
    let device = if flags & 2 != 0 {
        "mps"
    } else if flags & 8 != 0 {
        "vulkan"
    } else if flags & 1 != 0 {
        "cpu"
    } else {
        return Err("unsupported_device: no qualified audio device".into());
    };
    Ok(serde_json::json!({"device":device}).to_string())
}
