use super::contracts::RestorePackageRequest;

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
        (
            request.creation_workflow_runs.len(),
            100_000,
            "Creation Workflow runs",
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
    Ok(())
}
