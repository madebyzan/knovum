use serde::{Deserialize, Serialize};

use crate::command_error::CommandError;
use crate::local_resources::models::catalog::{self, ModelKind};

#[derive(Clone, Deserialize, Serialize, ts_rs::TS)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum ResourcePreparationTarget {
    Transcription,
    TextEmbedding,
    MediaImportTools,
}

impl ResourcePreparationTarget {
    pub fn models(&self) -> Option<&'static [ModelKind]> {
        match self {
            Self::Transcription => Some(catalog::TRANSCRIPTION_MODELS),
            Self::TextEmbedding => Some(catalog::TEXT_EMBEDDING_MODELS),
            Self::MediaImportTools => None,
        }
    }
}

#[derive(Clone, Copy, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub enum ResourcePreparationStage {
    Queued,
    Downloading,
    Verifying,
    Installing,
    Ready,
}

#[derive(Clone, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct ResourcePreparationProgress {
    pub stage: ResourcePreparationStage,
    pub downloaded: u64,
    pub total: Option<u64>,
    pub percent: Option<u8>,
}

#[derive(Clone, Copy, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub enum ResourcePreparationState {
    Queued,
    Preparing,
    Ready,
    Failed,
    Cancelled,
}

#[derive(Clone, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct ResourcePreparationSnapshot {
    pub job_id: String,
    pub title: String,
    pub target: ResourcePreparationTarget,
    pub started_at: u64,
    pub completed_at: Option<u64>,
    pub state: ResourcePreparationState,
    pub progress: ResourcePreparationProgress,
    pub error: Option<CommandError>,
}
