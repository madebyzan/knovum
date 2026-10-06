use serde::Serialize;
use sha2::{Digest, Sha256};

use super::catalog::{self, ModelKind};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub(crate) enum SnapshotKind {
    FasterWhisper,
    MlxWhisper,
    ImageSegmentation,
    TextEmbedding,
    VoxCpm2,
    SpeechAlignment,
    SpeechSeparation,
}

#[derive(Clone, Serialize, schemars::JsonSchema)]
pub(crate) struct SnapshotFile {
    pub path: &'static str,
    pub sha256: &'static str,
    pub size: u64,
}

impl SnapshotKind {
    pub(crate) const fn name(self) -> &'static str {
        match self {
            Self::FasterWhisper => "faster-whisper-small",
            Self::MlxWhisper => "mlx-whisper-small",
            Self::ImageSegmentation => "u2netp",
            Self::TextEmbedding => "qwen3-embedding",
            Self::VoxCpm2 => "voxcpm2",
            Self::SpeechAlignment => "qwen3-forced-aligner-0.6b",
            Self::SpeechSeparation => "spleeter-2stems-fp16",
        }
    }

    pub(crate) const fn models(self) -> &'static [ModelKind] {
        match self {
            Self::FasterWhisper => catalog::FASTER_WHISPER_MODELS,
            Self::MlxWhisper => catalog::MLX_WHISPER_MODELS,
            Self::ImageSegmentation => catalog::IMAGE_SEGMENTATION_MODELS,
            Self::TextEmbedding => catalog::TEXT_EMBEDDING_MODELS,
            Self::VoxCpm2 => catalog::VOXCPM2_MODELS,
            Self::SpeechAlignment => catalog::QWEN3_FORCED_ALIGNER_MODELS,
            Self::SpeechSeparation => catalog::SPEECH_SEPARATION_MODELS,
        }
    }

    pub(super) fn files(self) -> Vec<SnapshotFile> {
        self.models()
            .iter()
            .map(|kind| {
                let file = catalog::descriptor(*kind);
                SnapshotFile {
                    path: file.file,
                    sha256: file.sha256,
                    size: file.download_size,
                }
            })
            .collect()
    }

    pub(crate) fn total_bytes(self) -> u64 {
        self.models()
            .iter()
            .map(|kind| catalog::descriptor(*kind).download_size)
            .sum()
    }

    pub(crate) fn digest(self) -> String {
        let mut digest = Sha256::new();
        for file in self.files() {
            digest.update(file.path.as_bytes());
            digest.update([0]);
            digest.update(file.sha256.as_bytes());
            digest.update(file.size.to_le_bytes());
        }
        format!("{:x}", digest.finalize())
    }
}
