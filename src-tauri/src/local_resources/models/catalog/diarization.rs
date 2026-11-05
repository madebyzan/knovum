use serde::{Deserialize, Serialize};

use super::{ModelDescriptor, ModelKind, SnapshotKind};

const SEGMENTATION_ROOT: &str = "https://huggingface.co/csukuangfj/sherpa-onnx-pyannote-segmentation-3-0/resolve/9403a6902bb58e3d5ae8c7e77c3422de279db2e0";
const EMBEDDING_ROOT: &str = "https://huggingface.co/csukuangfj/speaker-embedding-models/resolve/0743f301363dec56491a490f6d6cbc9d67f9a3bf";

#[derive(
    Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord, Hash, ts_rs::TS,
)]
#[serde(rename_all = "camelCase")]
pub enum DiarizationVariant {
    Int8,
    Campplus,
    Full,
    Eres2netLarge,
}

#[derive(Clone, Copy, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord, Hash, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub enum DiarizationFile {
    Segmentation,
    Embedding,
    SegmentationLicense,
    EmbeddingLicense,
}

#[derive(Clone, Copy, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord, Hash, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DiarizationPart {
    pub variant: DiarizationVariant,
    pub file: DiarizationFile,
}

const fn parts(variant: DiarizationVariant) -> [ModelKind; 4] {
    [
        ModelKind::DiarizationPart(DiarizationPart {
            variant,
            file: DiarizationFile::Segmentation,
        }),
        ModelKind::DiarizationPart(DiarizationPart {
            variant,
            file: DiarizationFile::Embedding,
        }),
        ModelKind::DiarizationPart(DiarizationPart {
            variant,
            file: DiarizationFile::SegmentationLicense,
        }),
        ModelKind::DiarizationPart(DiarizationPart {
            variant,
            file: DiarizationFile::EmbeddingLicense,
        }),
    ]
}

const INT8_MODELS: [ModelKind; 4] = parts(DiarizationVariant::Int8);
const CAMPPLUS_MODELS: [ModelKind; 4] = parts(DiarizationVariant::Campplus);
const FULL_MODELS: [ModelKind; 4] = parts(DiarizationVariant::Full);
const ERES2NET_LARGE_MODELS: [ModelKind; 4] = parts(DiarizationVariant::Eres2netLarge);

impl DiarizationVariant {
    pub(crate) const fn models(self) -> &'static [ModelKind] {
        match self {
            Self::Int8 => &INT8_MODELS,
            Self::Campplus => &CAMPPLUS_MODELS,
            Self::Full => &FULL_MODELS,
            Self::Eres2netLarge => &ERES2NET_LARGE_MODELS,
        }
    }

    pub(crate) const fn spec_id(self) -> &'static str {
        match self {
            Self::Int8 => "sherpa-diarization-cpu-v1",
            Self::Campplus => "sherpa-diarization-campplus-cpu-v1",
            Self::Full => "sherpa-diarization-fp32-cpu-v1",
            Self::Eres2netLarge => "sherpa-diarization-eres2net-large-cpu-v1",
        }
    }

    pub(crate) const fn revision(self) -> &'static str {
        match self {
            Self::Int8 => "sherpa-pyannote3-int8-eres2net-base-v1",
            Self::Campplus => "sherpa-pyannote3-int8-campplus-v1",
            Self::Full => "sherpa-pyannote3-fp32-eres2net-base-v1",
            Self::Eres2netLarge => "sherpa-pyannote3-fp32-eres2net-large-v1",
        }
    }
}

impl DiarizationPart {
    pub(crate) const fn descriptor(self) -> ModelDescriptor {
        let (file, sha256, download_size) = match (self.variant, self.file) {
            (
                DiarizationVariant::Int8 | DiarizationVariant::Campplus,
                DiarizationFile::Segmentation,
            ) => (
                "segmentation.onnx",
                "d582f4b4c6b48205de7e0643c57df0df5615a3c176189be3fc461e9d18827b5d",
                1_540_506,
            ),
            (
                DiarizationVariant::Full | DiarizationVariant::Eres2netLarge,
                DiarizationFile::Segmentation,
            ) => (
                "segmentation.onnx",
                "220ad67ca923bef2fa91f2390c786097bf305bceb5e261d4af67b38e938e1079",
                5_992_913,
            ),
            (DiarizationVariant::Campplus, DiarizationFile::Embedding) => (
                "embedding.onnx",
                "aa3cfc16963a10586a9393f5035d6d6b57e98d358b347f80c2a30bf4f00ceba2",
                28_281_164,
            ),
            (DiarizationVariant::Eres2netLarge, DiarizationFile::Embedding) => (
                "embedding.onnx",
                "19547e85b6c14ec44b8add4e7cb9ce353c7e995d4f1c9ffd408176ac3a2d6895",
                116_058_710,
            ),
            (_, DiarizationFile::Embedding) => (
                "embedding.onnx",
                "1a331345f04805badbb495c775a6ddffcdd1a732567d5ec8b3d5749e3c7a5e4b",
                39_593_761,
            ),
            (_, DiarizationFile::SegmentationLicense) => (
                "LICENSE.segmentation",
                "14d7016ad68e7394d6e6b78d96cc2ae431c905287b89674cfdf021e79e62b8ba",
                1_061,
            ),
            (_, DiarizationFile::EmbeddingLicense) => (
                "LICENSE.embedding",
                "c71d239df91726fc519c6eb72d318ec65820627232b2f796219e87dcf35d0ab4",
                11_357,
            ),
        };
        ModelDescriptor {
            snapshot: SnapshotKind::Diarization(self.variant),
            file,
            sha256,
            download_size,
        }
    }

    pub(crate) fn url(self) -> String {
        match self.file {
            DiarizationFile::Segmentation => format!(
                "{SEGMENTATION_ROOT}/{}",
                match self.variant {
                    DiarizationVariant::Int8 | DiarizationVariant::Campplus => "model.int8.onnx",
                    DiarizationVariant::Full | DiarizationVariant::Eres2netLarge => "model.onnx",
                }
            ),
            DiarizationFile::Embedding => format!(
                "{EMBEDDING_ROOT}/{}",
                match self.variant {
                    DiarizationVariant::Campplus => {
                        "3dspeaker_speech_campplus_sv_zh_en_16k-common_advanced.onnx"
                    }
                    DiarizationVariant::Eres2netLarge => {
                        "3dspeaker_speech_eres2net_large_sv_zh-cn_3dspeaker_16k.onnx"
                    }
                    DiarizationVariant::Int8 | DiarizationVariant::Full => {
                        "3dspeaker_speech_eres2net_base_sv_zh-cn_3dspeaker_16k.onnx"
                    }
                }
            ),
            DiarizationFile::SegmentationLicense => format!("{SEGMENTATION_ROOT}/LICENSE"),
            DiarizationFile::EmbeddingLicense => {
                "https://raw.githubusercontent.com/modelscope/3D-Speaker/065629c313eaf1a01c65c640c46d77e61e9607b4/LICENSE".into()
            }
        }
    }
}
