pub const MAX_RELATION_RATIONALE_CHARS: usize = 16_000;
pub const MAX_PROVENANCE_QUOTE_CHARS: usize = 64_000;
pub const MAX_CHANGE_SET_OPERATIONS: usize = 512;

pub fn validate_event_entity_kind(kind: &str) -> Result<(), String> {
    match kind {
        "node" | "relation" | "provenance" | "type" => Ok(()),
        _ => Err("Unsupported knowledge event entity kind".into()),
    }
}

fn validate_event_kind(kind: &str) -> Result<(), String> {
    match kind {
        "created" | "updated" | "drafted" | "accepted" | "contested" | "superseded"
        | "invalidated" | "opened" | "resolved" | "restored" | "applied" | "rejected" | "moved" => {
            Ok(())
        }
        _ => Err("Unsupported knowledge event kind".into()),
    }
}

pub fn validate_event_pair(entity_kind: &str, event_kind: &str) -> Result<(), String> {
    validate_event_entity_kind(entity_kind)?;
    validate_event_kind(event_kind)?;
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

pub fn validate_relation_rationale(value: &str) -> Result<(), String> {
    validate_text_limit(value, MAX_RELATION_RATIONALE_CHARS, "Relation rationale")
}

pub fn validate_provenance_quote(value: &str) -> Result<(), String> {
    validate_text_limit(value, MAX_PROVENANCE_QUOTE_CHARS, "Provenance quote")
}

pub fn validate_text_limit(value: &str, max_chars: usize, label: &str) -> Result<(), String> {
    if value.encode_utf16().count() > max_chars {
        return Err(format!(
            "{label} exceeds the maximum of {max_chars} characters"
        ));
    }
    Ok(())
}
