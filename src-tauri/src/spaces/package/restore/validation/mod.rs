use super::contracts::RestorePackageRequest;
use tokio_util::sync::CancellationToken;

mod references;
mod speech;
mod translation;

pub(super) fn validate_request_shape_cancellable(
    request: &RestorePackageRequest,
    cancellation: &CancellationToken,
) -> Result<(), crate::app_error::AppError> {
    super::super::control::check(cancellation)?;
    let space = &request.space;
    if space.kind != "space"
        || space.title.trim().is_empty()
        || space.title.encode_utf16().count() > 512
    {
        return Err("Package Space metadata is invalid".into());
    }
    if request
        .run_steps
        .iter()
        .any(|step| step.node_kind == "speech_voice_candidate")
        || request.runs.iter().any(|run| {
            run.input_snapshot
                .get("workflow")
                .and_then(serde_json::Value::as_str)
                == Some("speech_voice_candidate_v1")
        })
    {
        return Err("Application voice generation does not belong in a Space package".into());
    }
    let limits = [
        (request.nodes.len(), 100_000usize, "nodes"),
        (request.structure_nodes.len(), 100_000, "structure nodes"),
        (request.layouts.len(), 100_000, "structure layouts"),
        (request.relation_styles.len(), 100_000, "relation styles"),
        (request.scenes.len(), 10_000, "Saved Views"),
        (request.relations.len(), 200_000, "Relations"),
        (request.derivations.len(), 100_000, "knowledge derivations"),
        (request.runs.len(), 100_000, "Runs"),
        (request.run_steps.len(), 1_000_000, "Run steps"),
        (request.run_attempts.len(), 2_000_000, "Run attempts"),
        (request.run_events.len(), 5_000_000, "Run events"),
        (
            request.external_agent_conversations.len(),
            100_000,
            "external Agent conversations",
        ),
        (
            request.external_agent_turns.len(),
            1_000_000,
            "external Agent turns",
        ),
        (request.conversations.len(), 100_000, "Conversations"),
        (
            request.conversation_messages.len(),
            1_000_000,
            "Conversation messages",
        ),
        (request.chunks.len(), 1_000_000, "knowledge chunks"),
        (request.object_types.len(), 10_000, "Knowledge Types"),
        (request.provenance.len(), 1_000_000, "provenance records"),
        (
            request.knowledge_events.len(),
            2_000_000,
            "knowledge events",
        ),
        (request.voice_profiles.len(), 100_000, "voice profiles"),
        (request.voice_revisions.len(), 1_000_000, "voice revisions"),
        (request.speech_projects.len(), 100_000, "speech projects"),
        (
            request.speech_project_revisions.len(),
            1_000_000,
            "speech project revisions",
        ),
        (request.speech_segments.len(), 2_000_000, "speech segments"),
        (request.speech_renders.len(), 5_000_000, "speech renders"),
        (
            request.speech_source_separations.len(),
            100_000,
            "speech source separations",
        ),
    ];
    if let Some((_, _, label)) = limits.into_iter().find(|(actual, limit, _)| actual > limit) {
        return Err(format!("Package contains too many {label}").into());
    }
    references::validate(request)?;
    super::super::control::check(cancellation)?;
    speech::validate(request)?;
    super::super::control::check(cancellation)?;
    Ok(translation::validate(request)?)
}

#[cfg(test)]
pub(super) fn validate_request_shape(
    request: &RestorePackageRequest,
) -> Result<(), crate::app_error::AppError> {
    validate_request_shape_cancellable(request, &CancellationToken::new())
}
