use crate::spaces::knowledge::contracts::KnowledgeActorKind;

pub(crate) fn validate_actor_kind(kind: &str) -> Result<(), String> {
    KnowledgeActorKind::parse(kind).map(|_| ())
}

pub(crate) fn validate_entity_kind(kind: &str) -> Result<(), String> {
    match kind {
        "node" | "relation" | "provenance" | "type" => Ok(()),
        _ => Err("Unsupported knowledge event entity kind".into()),
    }
}

pub(crate) fn validate_pair(entity_kind: &str, event_kind: &str) -> Result<(), String> {
    validate_entity_kind(entity_kind)?;
    let valid = match entity_kind {
        "node" => matches!(
            event_kind,
            "created"
                | "updated"
                | "drafted"
                | "accepted"
                | "contested"
                | "superseded"
                | "invalidated"
                | "opened"
                | "resolved"
                | "restored"
                | "moved"
        ),
        "relation" => matches!(
            event_kind,
            "created" | "updated" | "invalidated" | "restored" | "moved"
        ),
        "provenance" => matches!(
            event_kind,
            "created" | "superseded" | "invalidated" | "moved"
        ),
        "type" => matches!(event_kind, "created" | "updated"),
        _ => false,
    };
    valid.then_some(()).ok_or_else(|| {
        format!("Knowledge event '{event_kind}' is not valid for entity kind '{entity_kind}'")
    })
}
