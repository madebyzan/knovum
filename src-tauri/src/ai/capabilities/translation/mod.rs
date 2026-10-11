mod protocol;
#[cfg(test)]
mod tests;

use crate::ai::tasks::completion::TaskInput;
use crate::foundation::app_error::AppResult;

pub(crate) fn input(text: String, target_language: String) -> AppResult<TaskInput> {
    let language = protocol::language_name(&target_language)?;
    Ok(TaskInput {
        purpose: crate::ai::usage::UsagePurpose::Translation,
        system_prompt: format!("Translate the user text into {language}. Preserve meaning, structure, formatting, and proper nouns. Return only the translation."),
        instruction: String::new(), text, max_tokens: None, image: None,
    })
}

pub(crate) fn validate_target(target_language: &str) -> AppResult<()> {
    protocol::language_name(target_language).map(|_| ())
}
