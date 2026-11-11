use crate::local_resources::models::catalog::ImageModelCatalog;
use serde::Serialize;
use sha2::{Digest, Sha256};

use super::catalog::{self, ModelKind};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub(crate) enum SnapshotKind {
    Image(catalog::ImageModelVariant),
    Diarization(catalog::DiarizationVariant),
    Whisper(catalog::WhisperVariant),
    Separation(catalog::SeparationVariant),
    ImageSegmentation,
    ImageSegmentationSilueta,
    ImageSegmentationHuman,
    ImageSegmentationQuality,
    TextEmbedding,
    MossNano,
    VoxCpm2,
    VoxCpm2Bf16,
    VoxCpm2Q8,
    SpeechAlignment,
    SpeechAlignmentQ8,
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
            Self::Image(variant) => variant.name(),
            Self::Whisper(variant) => match variant {
                catalog::WhisperVariant::Tiny => "whisper-tiny-q8-bundled",
                catalog::WhisperVariant::Base => "whisper-base-q8-bundled",
                catalog::WhisperVariant::Small => "whisper-small-q8-bundled",
                catalog::WhisperVariant::Medium => "whisper-medium-q8-bundled",
                catalog::WhisperVariant::MediumFull => "whisper-medium-f16-bundled",
                catalog::WhisperVariant::LargeV3TurboQ8 => "whisper-large-v3-turbo-q8-bundled",
                catalog::WhisperVariant::LargeV3Turbo => "whisper-large-v3-turbo-f16-bundled",
                catalog::WhisperVariant::LargeV3 => "whisper-large-v3-f16-bundled",
            },
            Self::ImageSegmentation => "u2netp",
            Self::ImageSegmentationSilueta => "silueta",
            Self::ImageSegmentationHuman => "u2net-human-seg",
            Self::ImageSegmentationQuality => "u2net",
            Self::TextEmbedding => "qwen3-embedding",
            Self::MossNano => "moss-tts-nano",
            Self::VoxCpm2 => "voxcpm2",
            Self::VoxCpm2Bf16 => "voxcpm2-bf16",
            Self::VoxCpm2Q8 => "voxcpm2-q8_0",
            Self::SpeechAlignment => "qwen3-forced-aligner-0.6b-f16-bundled",
            Self::SpeechAlignmentQ8 => "qwen3-forced-aligner-0.6b-q8_0",
            Self::Separation(variant) => variant.snapshot_name(),
            Self::Diarization(variant) => variant.revision(),
        }
    }

    pub(crate) fn models(self) -> &'static [ModelKind] {
        match self {
            Self::Image(variant) => variant.models(),
            Self::Whisper(variant) => variant.models(),
            Self::ImageSegmentation => catalog::IMAGE_SEGMENTATION_MODELS,
            Self::ImageSegmentationSilueta => catalog::IMAGE_SEGMENTATION_SILUETA_MODELS,
            Self::ImageSegmentationHuman => catalog::IMAGE_SEGMENTATION_HUMAN_MODELS,
            Self::ImageSegmentationQuality => catalog::IMAGE_SEGMENTATION_QUALITY_MODELS,
            Self::TextEmbedding => catalog::TEXT_EMBEDDING_MODELS,
            Self::MossNano => catalog::MOSS_NANO_MODELS,
            Self::VoxCpm2 => catalog::VOXCPM2_MODELS,
            Self::VoxCpm2Bf16 => catalog::VOXCPM2_BF16_MODELS,
            Self::VoxCpm2Q8 => catalog::VOXCPM2_Q8_MODELS,
            Self::SpeechAlignment => catalog::QWEN3_FORCED_ALIGNER_MODELS,
            Self::SpeechAlignmentQ8 => catalog::QWEN3_ALIGNER_Q8_MODELS,
            Self::Separation(variant) => variant.models(),
            Self::Diarization(variant) => variant.models(),
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

    pub(crate) fn download_bytes(self) -> u64 {
        match self {
            Self::Image(variant) if variant.archive().is_some() => {
                let archive = variant
                    .archive()
                    .expect("Guard checked the release archive");
                let unpacked: u64 = self
                    .models()
                    .iter()
                    .filter(|kind| archive.owns(**kind))
                    .map(|kind| catalog::descriptor(*kind).download_size)
                    .sum();
                self.total_bytes() - unpacked + archive.bytes
            }
            _ => self.total_bytes(),
        }
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
        hex::encode(digest.finalize())
    }
}
