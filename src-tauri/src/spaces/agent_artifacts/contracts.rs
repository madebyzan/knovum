use serde::{Deserialize, Serialize};

use crate::ai::contracts::{AgentArtifact, AgentKind};

#[derive(Debug, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub struct RecoverableAgentCreation {
    pub session_id: String,
    pub artifacts: Vec<AgentArtifact>,
}

#[derive(Debug, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub struct AgentCreationRequest {
    pub kind: AgentKind,
    pub prompt: String,
    pub attachments: Vec<std::path::PathBuf>,
    pub space_id: String,
    pub session_id: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, ts_rs::TS)]
#[serde(
    tag = "type",
    rename_all = "snake_case",
    rename_all_fields = "camelCase"
)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub enum AgentCreationEvent {
    Thought {
        text: String,
    },
    Message {
        text: String,
    },
    ToolStarted {
        id: String,
        title: String,
    },
    ToolUpdated {
        id: String,
    },
    PermissionRequested {
        request_id: String,
        title: String,
        options: Vec<AgentPermissionOption>,
    },
}

#[derive(Debug, Clone, Deserialize, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub struct AgentPermissionOption {
    pub id: String,
    pub label: String,
    pub kind: String,
}
