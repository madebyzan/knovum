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

/// Forced alignment scales with the audio it aligns, so its budget does too. The budget covers
/// the alignment itself: loading the model is bounded by the worker's own startup budget.
pub(super) fn budget_ms(duration_ms: u64) -> u64 {
    duration_ms
        .saturating_mul(6)
        .clamp(120_000, 30 * 60 * 1_000)
}

pub(super) fn qualified_platform() -> Result<(), String> {
    #[cfg(knovum_supported_desktop)]
    return Ok(());
    #[cfg(not(knovum_supported_desktop))]
    Err("unsupported_device: Qwen3 forced alignment is not qualified for this platform".into())
}
