use super::{array_mut, IdMaps};
use serde_json::Value;

pub(super) fn remap(value: &mut Value, maps: &IdMaps) -> Result<(), String> {
    for row in array_mut(value, "derivations")? {
        maps.field(row, "outputNodeId", "nodes")?;
        for input in array_mut(row, "inputs")? {
            maps.optional(input, "nodeId", "nodes")?;
        }
    }
    for row in array_mut(value, "runs")? {
        for field in ["parentRunId", "retryOfRunId", "forkOfRunId"] {
            maps.optional(row, field, "runs")?;
        }
        if let Some(policy) = row.get_mut("policySnapshot").filter(|v| v.is_object()) {
            maps.optional(policy, "retryRootRunId", "runs")?;
        }
    }
    for row in array_mut(value, "runSteps")? {
        maps.field(row, "runId", "runs")?;
    }
    for row in array_mut(value, "runAttempts")? {
        maps.field(row, "runId", "runs")?;
        maps.field(row, "stepInstanceId", "runSteps")?;
    }
    for row in array_mut(value, "runEvents")? {
        maps.field(row, "runId", "runs")?;
        maps.optional(row, "stepInstanceId", "runSteps")?;
        maps.optional(row, "attemptId", "runAttempts")?;
    }
    for row in array_mut(value, "externalAgentConversations")? {
        maps.field(row, "runId", "runs")?;
        maps.field(row, "conversationId", "conversations")?;
    }
    for row in array_mut(value, "externalAgentTurns")? {
        maps.field(row, "conversationId", "externalAgentConversations")?;
        maps.optional(row, "messageId", "conversationMessages")?;
    }
    for row in array_mut(value, "conversationMessages")? {
        maps.field(row, "conversationId", "conversations")?;
        maps.optional(row, "parentMessageId", "conversationMessages")?;
    }
    Ok(())
}
