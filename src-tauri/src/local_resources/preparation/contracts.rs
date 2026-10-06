use serde::{Deserialize, Serialize};

use crate::app_error::AppError;
use crate::local_resources::models::catalog::{self, ModelKind};

#[derive(Clone, Deserialize, Serialize, ts_rs::TS)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum ResourcePreparationTarget {
    Transcription,
    TextEmbedding,
    ImageSegmentation,
    SpeechSynthesis,
    SpeechAlignment,
    SpeechSeparation,
    MediaImportTools,
    PythonRuntime {
        profile: crate::local_resources::runtimes::python::RuntimeProfile,
    },
}

impl ResourcePreparationTarget {
    pub fn models(&self) -> Option<&'static [ModelKind]> {
        match self {
            Self::Transcription => {
                (!catalog::TRANSCRIPTION_MODELS.is_empty()).then_some(catalog::TRANSCRIPTION_MODELS)
            }
            Self::TextEmbedding => Some(catalog::TEXT_EMBEDDING_MODELS),
            Self::ImageSegmentation => Some(catalog::IMAGE_SEGMENTATION_MODELS),
            Self::SpeechSynthesis => None,
            Self::SpeechAlignment => (!catalog::SPEECH_ALIGNMENT_MODELS.is_empty())
                .then_some(catalog::SPEECH_ALIGNMENT_MODELS),
            Self::SpeechSeparation => Some(catalog::SPEECH_SEPARATION_MODELS),
            Self::MediaImportTools | Self::PythonRuntime { .. } => None,
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
    RuntimeDownloading,
    RuntimeVerifying,
    RuntimeInstalling,
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
    pub error: Option<AppError>,
}
