mod protocol;
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
