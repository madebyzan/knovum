use super::contracts::RestorePackageRequest;

mod references;

pub(super) fn validate_request_shape(request: &RestorePackageRequest) -> Result<(), String> {
    let space = &request.space;
    if space.kind != "space"
        || space.title.trim().is_empty()
        || space.title.encode_utf16().count() > 512
    {
        return Err("Package Space metadata is invalid".into());
    }
    let limits = [
        (request.nodes.len(), 100_000usize, "nodes"),
        (request.structure_nodes.len(), 100_000, "structure nodes"),
        (request.layouts.len(), 100_000, "structure layouts"),
        (request.relation_styles.len(), 100_000, "relation styles"),
        (request.scenes.len(), 10_000, "Saved Views"),
        (request.relations.len(), 200_000, "Relations"),
        (request.derivations.len(), 100_000, "knowledge derivations"),
        (request.workflow_drafts.len(), 10_000, "Workflow drafts"),
        (
            request.workflow_versions.len(),
            100_000,
            "Workflow versions",
        ),
        (request.runs.len(), 100_000, "Runs"),
        (request.run_steps.len(), 1_000_000, "Run steps"),
        (request.run_batch_items.len(), 2_000_000, "Run batch items"),
        (request.run_attempts.len(), 2_000_000, "Run attempts"),
        (request.run_events.len(), 5_000_000, "Run events"),
        (
            request.automation_triggers.len(),
            100_000,
            "Automation triggers",
        ),
        (
            request.automation_occurrences.len(),
            1_000_000,
            "Automation occurrences",
        ),
        (
            request.external_tool_bindings.len(),
            100_000,
            "external tool bindings",
        ),
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
        (
            request.external_agent_messages.len(),
            2_000_000,
            "external Agent messages",
        ),
        (
            request.assistant_conversations.len(),
            100_000,
            "Assistant conversations",
        ),
        (
            request.assistant_messages.len(),
            1_000_000,
            "Assistant messages",
        ),
        (request.chunks.len(), 1_000_000, "knowledge chunks"),
        (request.threads.len(), 100_000, "AI threads"),
        (request.messages.len(), 1_000_000, "AI messages"),
        (request.object_types.len(), 10_000, "Knowledge Types"),
        (request.provenance.len(), 1_000_000, "provenance records"),
        (
            request.knowledge_events.len(),
            2_000_000,
            "knowledge events",
        ),
    ];
    if let Some((_, _, label)) = limits.into_iter().find(|(actual, limit, _)| actual > limit) {
        return Err(format!("Package contains too many {label}"));
    }
    references::validate(request)
}
