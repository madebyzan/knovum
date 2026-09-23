use std::collections::HashSet;

use super::RestorePackageRequest;

pub(super) fn validate(request: &RestorePackageRequest) -> Result<(), String> {
    let workflow_versions = request
        .workflow_versions
        .iter()
        .map(|row| (row.workflow_id.as_str(), row.version))
        .collect::<HashSet<_>>();
    let run_ids = unique_ids(request.runs.iter().map(|row| row.id.as_str()), "Run")?;
    let step_ids = unique_ids(
        request.run_steps.iter().map(|row| row.id.as_str()),
        "Run step",
    )?;
    let attempt_ids = unique_ids(
        request.run_attempts.iter().map(|row| row.id.as_str()),
        "Run attempt",
    )?;
    let trigger_ids = unique_ids(
        request
            .automation_triggers
            .iter()
            .map(|row| row.id.as_str()),
        "Automation trigger",
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
    let external_conversation_ids = unique_ids(
        request
            .external_agent_conversations
            .iter()
            .map(|row| row.id.as_str()),
        "external Agent conversation",
    )?;
    let external_turn_ids = unique_ids(
        request
            .external_agent_turns
            .iter()
            .map(|row| row.id.as_str()),
        "external Agent turn",
    )?;
    unique_ids(
        request
            .external_agent_messages
            .iter()
            .map(|row| row.id.as_str()),
        "external Agent message",
    )?;
    unique_ids(
        request
            .automation_occurrences
            .iter()
            .map(|row| row.id.as_str()),
        "Automation occurrence",
    )?;
    unique_ids(
        request
            .external_tool_bindings
            .iter()
            .map(|row| row.id.as_str()),
        "external tool binding",
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
    let mut batch_keys = HashSet::new();
    for row in &request.run_batch_items {
        if !batch_keys.insert((&row.lineage_run_id, &row.definition_node_id, &row.item_key)) {
            return Err("Package Run batch item keys must be unique".into());
        }
        require_reference(&run_ids, &row.lineage_run_id, "Run batch lineage")?;
        require_reference(&run_ids, &row.latest_run_id, "Run batch owner")?;
        require_reference(&step_ids, &row.latest_step_instance_id, "Run batch step")?;
        if let Some(id) = &row.child_run_id {
            require_reference(&run_ids, id, "Run batch child")?
        }
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
    for row in &request.automation_triggers {
        if !workflow_versions.contains(&(row.workflow_id.as_str(), row.workflow_version)) {
            return Err("Package Automation references an unavailable Workflow version".into());
        }
    }
    for row in &request.automation_occurrences {
        require_reference(
            &trigger_ids,
            &row.trigger_id,
            "Automation occurrence trigger",
        )?;
        if let Some(id) = &row.run_id {
            require_reference(&run_ids, id, "Automation occurrence Run")?
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
    }
    for row in &request.external_agent_turns {
        require_reference(
            &external_conversation_ids,
            &row.conversation_id,
            "external Agent turn conversation",
        )?;
    }
    for row in &request.external_agent_messages {
        require_reference(
            &external_conversation_ids,
            &row.conversation_id,
            "external Agent message conversation",
        )?;
        require_reference(
            &external_turn_ids,
            &row.turn_id,
            "external Agent message turn",
        )?;
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
