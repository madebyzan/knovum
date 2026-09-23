use crate::ai::{
    capabilities::chat,
    error::{AiError, AiResult},
    models::{
        CompletionOutput, CompletionRequest, CompletionStreamEvent, EditorTransformAction,
        EditorTransformRequest,
    },
};

const EDITOR_SYSTEM: &str = "You are Knovum's Markdown writing engine. Follow the requested transformation exactly. Preserve factual meaning, Markdown structure, links, code spans, and proper nouns unless the action explicitly requires a change. Return only the transformed text without commentary, labels, quotation marks, or code fences.";

pub async fn transform(request: EditorTransformRequest) -> AiResult<CompletionOutput> {
    let inst = instruction(request.action);
    let source = request.text.trim();
    if source.is_empty() {
        return Err(AiError::invalid(
            "Editor transform requires non-empty source text.",
        ));
    }
    let req = CompletionRequest {
        purpose: crate::ai::usage::UsagePurpose::Editor,
        target: request.target,
        system_prompt: EDITOR_SYSTEM.to_string(),
        instruction: inst.to_string(),
        text: source.to_string(),
        max_tokens: None,
    };
    chat::complete(req).await
}

pub async fn transform_stream(
    request: EditorTransformRequest,
    events: tauri::ipc::Channel<CompletionStreamEvent>,
    cancellation: tokio_util::sync::CancellationToken,
) -> AiResult<CompletionOutput> {
    let inst = instruction(request.action);
    let source = request.text.trim();
    if source.is_empty() {
        return Err(AiError::invalid(
            "Editor transform requires non-empty source text.",
        ));
    }
    let req = CompletionRequest {
        purpose: crate::ai::usage::UsagePurpose::Editor,
        target: request.target,
        system_prompt: EDITOR_SYSTEM.to_string(),
        instruction: inst.to_string(),
        text: source.to_string(),
        max_tokens: None,
    };
    chat::complete_stream(req, events, cancellation).await
}

fn instruction(action: EditorTransformAction) -> &'static str {
    match action {
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
            "Continue the text naturally from where it stops. Match the existing language, voice, formatting, and level of detail. Return only the continuation, not the source text."
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
