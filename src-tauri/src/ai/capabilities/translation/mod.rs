mod protocol;
pub(crate) mod speech_rate;
#[cfg(test)]
mod tests;

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::ai::tasks::completion::TaskInput;
use crate::foundation::app_error::AppResult;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct BatchLineInput {
    pub id: usize,
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,
}

pub(crate) fn input(text: String, target_language: String) -> AppResult<TaskInput> {
    dubbing_input(text, target_language, None, &[])
}

pub(crate) fn dubbing_input(
    text: String,
    target_language: String,
    duration_ms: Option<u64>,
    terms: &[String],
) -> AppResult<TaskInput> {
    let language = protocol::language_name(&target_language)?;
    let term_clause = term_preservation_clause(terms);
    let prompt = match duration_ms {
        Some(ms) if ms > 0 => {
            let seconds = (ms as f64) / 1000.0;
            format!(
                "You are an expert dubbing localization adapter. Translate the dialogue into natural spoken {language}.{term_clause} \
                 This line must fit a physical speech duration window of {seconds:.2}s ({ms}ms). \
                 Adapt phrasing and syllable density according to natural spoken rhythm so that the target dialogue \
                 can be comfortably spoken at an authentic conversational cadence within this exact time window. \
                 Preserve character intent, emotional weight, proper nouns, and subtext. Return only the translated dialogue."
            )
        }
        _ => format!(
            "Translate the user text into {language}.{term_clause} Preserve meaning, structure, formatting, and proper nouns. Return only the translation."
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

pub(crate) fn dubbing_batch_input(
    lines: &[BatchLineInput],
    target_language: String,
    terms: &[String],
) -> AppResult<TaskInput> {
    let language = protocol::language_name(&target_language)?;
    let term_clause = term_preservation_clause(terms);
    let system_prompt = format!(
        "You are an expert dubbing localization adapter. Translate the dialogue lines into natural spoken {language}. \
         Maintain natural conversational flow, authentic rhythm, character intent, and emotional weight across the entire dialogue.{term_clause} \
         For lines specifying a physical duration limit (duration_ms), adapt the phrasing and syllable density so the line can be comfortably delivered within that window. \
         Format your response strictly as a JSON array of objects with the structure: [{{\"id\": <number>, \"translation\": \"<translated text>\"}}]. \
         Do NOT include markdown code fences, preambles, or commentary. Return only the JSON array."
    );
    let text = serde_json::to_string(lines).map_err(|error| {
        crate::foundation::app_error::AppError::new("json_error", error.to_string())
    })?;
    Ok(TaskInput {
        purpose: crate::ai::usage::UsagePurpose::Translation,
        system_prompt,
        instruction: String::new(),
        text,
        max_tokens: None,
        image: None,
    })
}

pub(crate) fn parse_batch_output(raw_output: &str) -> Result<HashMap<usize, String>, String> {
    let text = raw_output.trim();
    let text = text
        .strip_prefix("```json")
        .or_else(|| text.strip_prefix("```"))
        .unwrap_or(text);
    let text = text.strip_suffix("```").unwrap_or(text).trim();

    let start = text
        .find('[')
        .ok_or("No JSON array start found in batch translation output")?;
    let end = text
        .rfind(']')
        .ok_or("No JSON array end found in batch translation output")?;
    let slice = &text[start..=end];

    #[derive(Deserialize)]
    struct BatchLineOutput {
        id: usize,
        translation: String,
    }

    let parsed: Vec<BatchLineOutput> = serde_json::from_str(slice)
        .map_err(|error| format!("Failed to parse batch translation JSON: {error}"))?;

    let mut map = HashMap::with_capacity(parsed.len());
    for item in parsed {
        let clean = item.translation.trim().to_owned();
        if !clean.is_empty() {
            map.insert(item.id, clean);
        }
    }
    Ok(map)
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
    terms: &[String],
) -> AppResult<TaskInput> {
    let language = protocol::language_name(&target_language)?;
    let window_seconds = duration_ms as f64 / 1000.0;
    let spoken_seconds = previous_duration_ms as f64 / 1000.0;
    let reduction_percent = ((spoken_seconds - window_seconds) / spoken_seconds * 100.0).round();
    let term_clause = term_preservation_clause(terms);
    Ok(TaskInput {
        purpose: crate::ai::usage::UsagePurpose::Translation,
        system_prompt: format!(
            "You are an expert dubbing localization adapter. Your previous {language} adaptation is estimated to be spoken in {spoken_seconds:.2}s, \
             but its physical speech duration window is only {window_seconds:.2}s.{term_clause} \
             Rewrite the adaptation roughly {reduction_percent:.0}% shorter in spoken duration so it can be delivered at a natural conversational \
             cadence within the window. Keep character intent, emotional weight, proper nouns, and subtext as far as the shorter form allows. \
             Return only the shortened adaptation."
        ),
        instruction: format!("Previous adaptation:\n{previous_translation}"),
        text,
        max_tokens: None,
        image: None,
    })
}

fn term_preservation_clause(terms: &[String]) -> String {
    let filtered: Vec<&str> = terms
        .iter()
        .map(|term| term.trim())
        .filter(|term| !term.is_empty())
        .collect();
    if filtered.is_empty() {
        String::new()
    } else {
        format!(
            " Do NOT translate, transliterate, or alter the following proper nouns and terms; preserve them exactly as written: [{}].",
            filtered.join(", ")
        )
    }
}
