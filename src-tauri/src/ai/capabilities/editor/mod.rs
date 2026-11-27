pub(crate) mod media;

use self::media::EditorMediaCapabilities;
use crate::ai::{
    models::EditorTransformAction,
    tasks::{
        completion::TaskInput,
        text::{EditorDocumentContext, EditorTransformCommandRequest},
    },
};
use crate::foundation::app_error::{AppError, AppResult};

const EDITOR_SYSTEM: &str = "You are Knovum's Markdown writing engine. Follow the requested transformation or instruction exactly. Write clean, well-structured GitHub Flavored Markdown: use headings (`#`, `##`), bullet/numbered/task lists, tables, callouts (`> [!NOTE]`), KaTeX math (`$...$` / `$$...$$`), and fenced code blocks (including ```mermaid for flowcharts/diagrams and language-tagged fences for code) whenever structure, code, or diagrams fit the request or match the surrounding document. Preserve factual meaning, grounded citations like [C1], links, code spans, and proper nouns unless the request explicitly asks to change them. Return only the resulting Markdown content without preamble, labels, surrounding quotes, or an outer ```markdown wrapper around the entire reply.";

const MAX_INSTRUCTION_CHARS: usize = 2_000;
const MAX_CONTEXT_TEXT_CHARS: usize = 2_500;
const MAX_PREVIOUS_DRAFT_CHARS: usize = 16_000;
const MAX_REFERENCES: usize = 5;
const MAX_REFERENCE_EXCERPT_CHARS: usize = 2_000;

pub(crate) fn input(
    request: &EditorTransformCommandRequest,
    media_capabilities: EditorMediaCapabilities,
) -> AppResult<TaskInput> {
    let source_text = request.text.trim();
    let custom_instruction = normalized_optional(request.instruction.as_deref());
    let previous_draft = normalized_optional(request.previous_draft.as_deref());
    let context = request.context.as_ref();

    validate_bounds(custom_instruction, previous_draft, context)?;

    match request.action {
        EditorTransformAction::Custom => {
            if custom_instruction.is_none() {
                return Err(AppError::invalid(
                    "Custom editor instruction requires non-empty prompt text",
                ));
            }
        }
        EditorTransformAction::Complete => {
            let has_preceding = context
                .and_then(|ctx| normalized_optional(ctx.preceding_text.as_deref()))
                .is_some();
            if source_text.is_empty() && !has_preceding {
                return Err(AppError::invalid(
                    "Editor continuation requires preceding document context",
                ));
            }
        }
        _ => {
            if source_text.is_empty() {
                return Err(AppError::invalid(
                    "Editor transform requires non-empty source text",
                ));
            }
        }
    }

    let instruction = build_instruction(request.action, custom_instruction, previous_draft);
    let payload = build_payload(source_text, previous_draft, context);
    let system_prompt = format!("{EDITOR_SYSTEM}{}", media_capabilities.prompt_guidance());

    Ok(TaskInput {
        purpose: crate::ai::usage::UsagePurpose::Editor,
        system_prompt,
        instruction,
        text: payload,
        max_tokens: None,
        image: None,
    })
}

fn normalized_optional(value: Option<&str>) -> Option<&str> {
    value.map(str::trim).filter(|text| !text.is_empty())
}

fn validate_bounds(
    instruction: Option<&str>,
    previous_draft: Option<&str>,
    context: Option<&EditorDocumentContext>,
) -> AppResult<()> {
    if instruction.is_some_and(|text| text.chars().count() > MAX_INSTRUCTION_CHARS) {
        return Err(AppError::invalid(
            "Editor instruction exceeds maximum length",
        ));
    }
    if previous_draft.is_some_and(|text| text.chars().count() > MAX_PREVIOUS_DRAFT_CHARS) {
        return Err(AppError::invalid("Previous draft exceeds maximum length"));
    }
    let Some(context) = context else {
        return Ok(());
    };
    let references = context.references.as_deref().unwrap_or(&[]);
    if references.len() > MAX_REFERENCES {
        return Err(AppError::invalid("Too many Space references attached"));
    }
    for field in [
        context.preceding_text.as_deref(),
        context.following_text.as_deref(),
    ] {
        if field.is_some_and(|text| text.chars().count() > MAX_CONTEXT_TEXT_CHARS) {
            return Err(AppError::invalid("Document context exceeds maximum length"));
        }
    }
    for reference in references {
        if reference.node_id.trim().is_empty() {
            return Err(AppError::invalid("Reference node identifier is required"));
        }
        if reference.excerpt.chars().count() > MAX_REFERENCE_EXCERPT_CHARS {
            return Err(AppError::invalid(
                "Reference excerpt exceeds maximum length",
            ));
        }
    }
    Ok(())
}

fn build_instruction(
    action: EditorTransformAction,
    custom: Option<&str>,
    previous_draft: Option<&str>,
) -> String {
    let base = match action {
        EditorTransformAction::Custom => custom.unwrap_or_default(),
        other => preset_instruction(other),
    };
    match (action, custom, previous_draft) {
        (EditorTransformAction::Custom, _, Some(_)) => {
            format!("Refine the previous draft according to this follow-up instruction: {base}")
        }
        (EditorTransformAction::Custom, _, None) => base.to_owned(),
        (_, Some(refinement), Some(_)) => {
            format!("{base}\nApply this follow-up adjustment to the previous draft: {refinement}")
        }
        (_, Some(refinement), None) => format!("{base}\nAdditional guidance: {refinement}"),
        (_, None, _) => base.to_owned(),
    }
}

fn build_payload(
    source_text: &str,
    previous_draft: Option<&str>,
    context: Option<&EditorDocumentContext>,
) -> String {
    let mut sections = Vec::new();
    if let Some(ctx) = context {
        let mut meta = Vec::new();
        if let Some(title) = normalized_optional(ctx.document_title.as_deref()) {
            meta.push(format!("Document title: {title}"));
        }
        if let Some(heading) = normalized_optional(ctx.section_heading.as_deref()) {
            meta.push(format!("Current section: {heading}"));
        }
        if !meta.is_empty() {
            sections.push(meta.join("\n"));
        }
        if let Some(before) = normalized_optional(ctx.preceding_text.as_deref()) {
            sections.push(format!("Preceding context:\n{before}"));
        }
        if let Some(after) = normalized_optional(ctx.following_text.as_deref()) {
            sections.push(format!("Following context:\n{after}"));
        }
        let references = ctx.references.as_deref().unwrap_or(&[]);
        if !references.is_empty() {
            let formatted = references
                .iter()
                .filter_map(|reference| {
                    let excerpt = reference.excerpt.trim();
                    if excerpt.is_empty() {
                        return None;
                    }
                    let title = reference.title.trim();
                    Some(format!("- [{title}]\n{excerpt}"))
                })
                .collect::<Vec<_>>();
            if !formatted.is_empty() {
                sections.push(format!("Space references:\n{}", formatted.join("\n\n")));
            }
        }
    }
    if !source_text.is_empty() {
        sections.push(format!("Target text:\n{source_text}"));
    }
    if let Some(draft) = previous_draft {
        sections.push(format!("Previous draft:\n{draft}"));
    }
    if sections.len() == 1
        && !source_text.is_empty()
        && context.is_none()
        && previous_draft.is_none()
    {
        return source_text.to_owned();
    }
    sections.join("\n\n")
}

fn preset_instruction(action: EditorTransformAction) -> &'static str {
    match action {
        EditorTransformAction::Custom => "",
        EditorTransformAction::Improve => {
            "Improve clarity, flow, precision, and word choice while preserving the original intent and approximate length."
        }
        EditorTransformAction::FixGrammar => {
            "Fix spelling, grammar, punctuation, capitalization, and obvious typographic mistakes. Do not rewrite wording that is already correct."
        }
        EditorTransformAction::Extend => {
            "Expand the text with useful detail and smoother transitions. Add substance, not filler, while keeping the same viewpoint and language."
        }
        EditorTransformAction::Reduce => {
            "Make the text materially shorter and denser while preserving every important fact, requirement, and conclusion."
        }
        EditorTransformAction::Simplify => {
            "Rewrite with simpler vocabulary and sentence structure while preserving technical accuracy and all important details."
        }
        EditorTransformAction::Emojify => {
            "Add a restrained number of contextually appropriate emoji that improve scanability. Do not make the text childish or change its meaning."
        }
        EditorTransformAction::Complete => {
            "Continue the document naturally from where the preceding context or target text stops. Match the existing language, voice, Markdown formatting, and level of detail. Return only the continuation, not the source text."
        }
        EditorTransformAction::Summarize => {
            "Summarize the text into a concise, information-dense version. Preserve decisions, constraints, numbers, names, and action structure."
        }
        EditorTransformAction::ToneAcademic => {
            "Rewrite in a rigorous academic tone: precise, neutral, structured, and evidence-oriented without unnecessary jargon."
        }
        EditorTransformAction::ToneBusiness => {
            "Rewrite in a concise professional business tone: clear, decisive, practical, and easy to scan."
        }
        EditorTransformAction::ToneCasual => {
            "Rewrite in a natural casual tone that feels relaxed and conversational without becoming sloppy."
        }
        EditorTransformAction::ToneConfident => {
            "Rewrite in a confident, decisive tone without exaggeration, arrogance, or unsupported claims."
        }
        EditorTransformAction::ToneCreative => {
            "Rewrite with more vivid, original phrasing and rhythm while preserving the underlying facts and intent."
        }
        EditorTransformAction::ToneFriendly => {
            "Rewrite in a warm, approachable, cooperative tone while remaining concise and professional."
        }
        EditorTransformAction::ToneHumorous => {
            "Rewrite with light, tasteful humor while preserving meaning and avoiding jokes that obscure important information."
        }
        EditorTransformAction::ToneInformative => {
            "Rewrite to maximize clarity and information density, making relationships and explanations explicit where useful."
        }
        EditorTransformAction::ToneInspirational => {
            "Rewrite in an encouraging, motivating tone while avoiding hype, clichés, or unsupported promises."
        }
        EditorTransformAction::ToneObjective => {
            "Rewrite in a neutral, impersonal, evidence-focused tone and remove subjective or emotionally loaded phrasing."
        }
        EditorTransformAction::TonePersuasive => {
            "Rewrite persuasively with a clear argument and strong structure, but do not invent evidence or overstate certainty."
        }
    }
}

#[cfg(test)]
mod tests;
