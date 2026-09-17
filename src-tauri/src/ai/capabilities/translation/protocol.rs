use crate::ai::error::{AiError, AiResult};

pub fn language_name(target: &str) -> AiResult<&'static str> {
    match target {
        "zh-CN" | "zh" | "zh-Hans" => Ok("Simplified Chinese"),
        "zh-TW" | "zh-HK" | "zh-Hant" => Ok("Traditional Chinese"),
        "en" | "en-US" | "en-GB" => Ok("English"),
        "ja" => Ok("Japanese"),
        "ko" => Ok("Korean"),
        "fr" => Ok("French"),
        "de" => Ok("German"),
        "es" => Ok("Spanish"),
        "pt" | "pt-BR" | "pt-PT" => Ok("Portuguese"),
        "it" => Ok("Italian"),
        "ru" => Ok("Russian"),
        "ar" => Ok("Arabic"),
        _ => Err(AiError::invalid(format!(
            "Unsupported translation target: {target}"
        ))),
    }
}
