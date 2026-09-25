use std::collections::HashSet;

use super::RestorePackageRequest;

pub(super) fn validate(request: &RestorePackageRequest) -> Result<(), String> {
    let run_ids = unique_ids(request.runs.iter().map(|row| row.id.as_str()), "Run")?;
    let step_ids = unique_ids(
        request.run_steps.iter().map(|row| row.id.as_str()),
        "Run step",
    )?;
    let attempt_ids = unique_ids(
        request.run_attempts.iter().map(|row| row.id.as_str()),
        "Run attempt",
    )?;
    let conversation_ids = unique_ids(
        request
            .assistant_conversations
            .iter()
            .map(|row| row.id.as_str()),
        "Assistant conversation",
    )?;
    let assistant_message_ids = unique_ids(
        request.assistant_messages.iter().map(|row| row.id.as_str()),
        "Assistant message",
    )?;
    if request
        .assistant_conversations
        .iter()
        .any(|row| row.active_branch_id.is_empty())
    {
        return Err("Package Assistant active branch must be nonempty".into());
    }
    let external_conversation_ids = unique_ids(
        request
            .external_agent_conversations
            .iter()
            .map(|row| row.id.as_str()),
        "external Agent conversation",
    )?;
    unique_ids(
        request
            .external_agent_turns
            .iter()
            .map(|row| row.id.as_str()),
        "external Agent turn",
    )?;

    for row in &request.runs {
        for reference in [
            &row.parent_run_id,
            &row.retry_of_run_id,
            &row.fork_of_run_id,
        ]
        .into_iter()
        .flatten()
        {
            require_reference(&run_ids, reference, "Run lineage")?;
        }
    }
    for row in &request.run_steps {
        require_reference(&run_ids, &row.run_id, "Run step owner")?;
    }
    for row in &request.run_attempts {
        require_reference(&run_ids, &row.run_id, "Run attempt owner")?;
        require_reference(&step_ids, &row.step_instance_id, "Run attempt step")?;
    }
    for row in &request.run_events {
        require_reference(&run_ids, &row.run_id, "Run event owner")?;
        if let Some(id) = &row.step_instance_id {
            require_reference(&step_ids, id, "Run event step")?
        }
        if let Some(id) = &row.attempt_id {
            require_reference(&attempt_ids, id, "Run event attempt")?
        }
    }
    for row in &request.assistant_messages {
        require_reference(
            &conversation_ids,
            &row.conversation_id,
            "Assistant message conversation",
        )?;
        if let Some(id) = &row.parent_message_id {
            require_reference(&assistant_message_ids, id, "Assistant parent message")?;
        }
    }
    for row in &request.external_agent_conversations {
        require_reference(&run_ids, &row.run_id, "external Agent conversation Run")?;
        require_reference(
            &conversation_ids,
            &row.assistant_conversation_id,
            "Assistant conversation",
        )?;
    }
    for row in &request.external_agent_turns {
        require_reference(
            &external_conversation_ids,
            &row.conversation_id,
            "external Agent turn conversation",
        )?;
        if let Some(id) = &row.assistant_message_id {
            require_reference(&assistant_message_ids, id, "Assistant message")?;
        }
    }
    Ok(())
}

fn unique_ids<'a>(
    ids: impl Iterator<Item = &'a str>,
    label: &str,
) -> Result<HashSet<&'a str>, String> {
    let mut unique = HashSet::new();
    for id in ids {
        if id.is_empty() || !unique.insert(id) {
            return Err(format!("Package {label} IDs must be nonempty and unique"));
        }
    }
    Ok(unique)
}

fn require_reference(ids: &HashSet<&str>, id: &str, label: &str) -> Result<(), String> {
    if ids.contains(id) {
        Ok(())
    } else {
        Err(format!("Package {label} reference is unavailable"))
    }
}
