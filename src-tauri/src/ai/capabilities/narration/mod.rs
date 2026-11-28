#[cfg(test)]
mod tests;

use crate::ai::tasks::{
    completion::TaskInput,
    text::{NarrationPreparationCommandRequest, NarrationPreparationMode},
};
use crate::foundation::app_error::{AppError, AppResult};

const MAX_NARRATION_INPUT_CHARS: usize = 32_000;

const SHARED_OUTPUT_RULES: &str = "\
Preserve the exact source language (do not translate). \
Output ONLY plain spoken prose ready for text-to-speech synthesis: \
use natural punctuation for breath and pacing, separate logical paragraphs with blank lines, \
and NEVER output Markdown syntax (#, **, *, -, >), numbered/bulleted lists, citation markers ([C1], [1], [^1]), \
URLs, emojis, bracketed stage directions, or any preamble/closing commentary.";

const CLEAN_SYSTEM_PROMPT: &str = "\
You are a professional audiobook and broadcast script editor preparing raw document text for text-to-speech narration. \
Preserve the author's original facts, arguments, wording, and tone as faithfully as possible. \
Never invent information or omit substantive points. \
Remove non-spoken document noise: running headers/footers, page numbers, broken PDF line wraps or hyphenation, \
table-of-contents leaders, copyright/navigation boilerplate, citation or footnote markers, parenthesized timestamps, \
bare URLs, raw code/table remnants, and purely visual references (such as 'see table below' or 'as shown in Figure 2'). \
Rewrite awkward written abbreviations, symbols, or compact metrics into natural spoken form when reading them literally would sound unnatural.";

const ADAPT_SYSTEM_PROMPT: &str = "\
You are a professional audiobook and broadcast script adapter transforming written documents, outlines, \
bullet lists, or rough notes into natural, engaging spoken narration for text-to-speech synthesis. \
Stay strictly grounded in the source material: never fabricate facts, statistics, or claims. \
Convert fragmented lists, slide points, and dense nested clauses into smooth, listenable spoken paragraphs \
with clear transitions and comfortable breath rhythm. \
Remove all non-spoken artifacts (headers, citations, URLs, visual-only references) and express shorthand or symbols in natural spoken form. \
Do NOT add generic podcast intros or outros (such as 'Hello everyone' or 'Thanks for listening').";

pub(crate) fn input(request: &NarrationPreparationCommandRequest) -> AppResult<TaskInput> {
    let source = request.text.trim();
    if source.is_empty() {
        return Err(AppError::invalid(
            "Narration preparation requires non-empty source text",
        ));
    }
    if source.chars().count() > MAX_NARRATION_INPUT_CHARS {
        return Err(AppError::invalid(
            "Narration source text exceeds the 32,000-character limit",
        ));
    }
    let role_prompt = match request.mode {
        NarrationPreparationMode::Clean => CLEAN_SYSTEM_PROMPT,
        NarrationPreparationMode::Adapt => ADAPT_SYSTEM_PROMPT,
    };
    Ok(TaskInput {
        purpose: crate::ai::usage::UsagePurpose::Editor,
        system_prompt: format!("{role_prompt}\n\n{SHARED_OUTPUT_RULES}"),
        instruction: String::new(),
        text: source.to_owned(),
        max_tokens: None,
        image: None,
    })
}
