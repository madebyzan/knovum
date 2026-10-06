pub(crate) mod control;
pub(crate) mod messages;
pub(crate) mod models;
pub(crate) mod tools;

pub(crate) const PROTOCOL_VERSION: u16 = 1;
pub(crate) const GRAPH_VERSION: &str = "knovum-assistant-v1/deepagents-0.7.22/langgraph-1.2.13";

// Collect the closed wire graph used by the Python contract generator.
#[allow(dead_code)]
#[derive(schemars::JsonSchema)]
pub(crate) struct AssistantWireSchemas {
    initialize: control::AssistantInitialize,
    ready: control::AssistantReady,
    prune_threads: control::AssistantPruneThreads,
    execute: control::AssistantExecute,
    execution_result: control::AssistantExecutionResult,
    control: control::AssistantControl,
    projection_request: messages::AssistantProjectionRequest,
    projection: messages::AssistantProjection,
    delta: messages::AssistantContentDelta,
    tool_invocation: tools::AssistantToolInvocation,
    tool_outcome: tools::AssistantToolOutcome,
    model_admission: models::AssistantModelAdmission,
    model_completion: models::AssistantModelCompletion,
    model_failure: models::AssistantModelFailure,
    tool_admission: tools::AssistantToolAdmission,
}
