use std::time::{SystemTime, UNIX_EPOCH};

pub(crate) fn validate_language(language: &str) -> Result<&'static str, String> {
    match language.trim().to_ascii_lowercase().as_str() {
        "zh" | "chinese" => Ok("zh"),
        "en" | "english" => Ok("en"),
        "yue" | "cantonese" => Ok("yue"),
        "fr" | "french" => Ok("fr"),
        "de" | "german" => Ok("de"),
        "it" | "italian" => Ok("it"),
        "pt" | "portuguese" => Ok("pt"),
        "ru" | "russian" => Ok("ru"),
        "es" | "spanish" => Ok("es"),
        _ => Err("unsupported_language: Qwen3 forced alignment does not support this transcript language".into()),
    }
}

pub(super) fn deadline(duration_ms: u64) -> Result<u64, String> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| error.to_string())?;
    let now_ms = u64::try_from(now.as_millis()).map_err(|error| error.to_string())?;
    now_ms
        .checked_add(
            duration_ms
                .saturating_mul(6)
                .clamp(120_000, 30 * 60 * 1_000),
        )
        .ok_or_else(|| "Alignment deadline exceeds the supported range".into())
}

pub(super) fn qualified_platform() -> Result<(), String> {
    #[cfg(any(
        all(target_os = "macos", target_arch = "aarch64"),
        all(target_os = "windows", target_arch = "x86_64")
    ))]
    return Ok(());
    #[cfg(not(any(
        all(target_os = "macos", target_arch = "aarch64"),
        all(target_os = "windows", target_arch = "x86_64")
    )))]
    Err("unsupported_device: Qwen3 forced alignment is not qualified for this platform".into())
}
