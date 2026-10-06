pub(crate) mod control;
pub(crate) mod messages;
pub(crate) mod models;
pub(crate) mod tools;

pub(crate) const PROTOCOL_VERSION: u16 = 2;
pub(crate) const GRAPH_VERSION: &str = "knovum-agent-v2/deepagents-0.7.22/langgraph-1.2.13";

// Collect the closed wire graph used by the Python contract generator.
#[allow(dead_code)]
#[derive(schemars::JsonSchema)]
pub(crate) struct AgentWireSchemas {
    initialize: control::AgentInitialize,
    ready: control::AgentReady,
    prune_threads: control::AgentPruneThreads,
    execute: control::AgentExecute,
    execution_result: control::AgentExecutionResult,
    control: control::AgentControl,
    projection_request: messages::AgentProjectionRequest,
    projection: messages::AgentProjection,
    delta: messages::AgentContentDelta,
    tool_invocation: tools::AgentToolInvocation,
    tool_outcome: tools::AgentToolOutcome,
    model_admission: models::AgentModelAdmission,
    model_completion: models::AgentModelCompletion,
    model_failure: models::AgentModelFailure,
    tool_admission: tools::AgentToolAdmission,
}
