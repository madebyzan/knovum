mod protocol;
pub(crate) mod speech_rate;
#[cfg(test)]
mod tests;

use crate::ai::tasks::completion::TaskInput;
use crate::foundation::app_error::AppResult;

pub(crate) fn input(text: String, target_language: String) -> AppResult<TaskInput> {
    dubbing_input(text, target_language, None)
}

pub(crate) fn dubbing_input(
    text: String,
    target_language: String,
    duration_ms: Option<u64>,
) -> AppResult<TaskInput> {
    let language = protocol::language_name(&target_language)?;
    let prompt = match duration_ms {
        Some(ms) if ms > 0 => {
            let seconds = (ms as f64) / 1000.0;
            format!(
                "You are an expert dubbing localization adapter. Translate the dialogue into natural spoken {language}. \
                 This line must fit a physical speech duration window of {seconds:.2}s ({ms}ms). \
                 Adapt phrasing and syllable density according to natural spoken rhythm so that the target dialogue \
                 can be comfortably spoken at an authentic conversational cadence within this exact time window. \
                 Preserve character intent, emotional weight, and subtext. Return only the translated dialogue."
            )
        }
        _ => format!(
            "Translate the user text into {language}. Preserve meaning, structure, formatting, and proper nouns. Return only the translation."
        ),
    };
    Ok(TaskInput {
        purpose: crate::ai::usage::UsagePurpose::Translation,
        system_prompt: prompt,
        instruction: String::new(),
        text,
        max_tokens: None,
        image: None,
    })
}

pub(crate) fn validate_target(target_language: &str) -> AppResult<()> {
    protocol::language_name(target_language).map(|_| ())
}

/// Builds the shortening round for an adaptation whose estimated spoken duration clearly exceeds
/// its physical window. `previous_duration_ms` must exceed `duration_ms`; callers gate on
/// [`speech_rate::requires_reduction`].
pub(crate) fn dubbing_reduction_input(
    text: String,
    target_language: String,
    duration_ms: u64,
    previous_translation: &str,
    previous_duration_ms: u64,
) -> AppResult<TaskInput> {
    let language = protocol::language_name(&target_language)?;
    let window_seconds = duration_ms as f64 / 1000.0;
    let spoken_seconds = previous_duration_ms as f64 / 1000.0;
    let reduction_percent = ((spoken_seconds - window_seconds) / spoken_seconds * 100.0).round();
    Ok(TaskInput {
        purpose: crate::ai::usage::UsagePurpose::Translation,
        system_prompt: format!(
            "You are an expert dubbing localization adapter. Your previous {language} adaptation is estimated to be spoken in {spoken_seconds:.2}s, \
             but its physical speech duration window is only {window_seconds:.2}s. \
             Rewrite the adaptation roughly {reduction_percent:.0}% shorter in spoken duration so it can be delivered at a natural conversational \
             cadence within the window. Keep character intent, emotional weight, and subtext as far as the shorter form allows. \
             Return only the shortened adaptation."
        ),
        instruction: format!("Previous adaptation:\n{previous_translation}"),
        text,
        max_tokens: None,
        image: None,
    })
}
