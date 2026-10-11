use super::{ModelKind, WhisperVariant};
use crate::local_resources::models::{snapshots::SnapshotKind, ModelDescriptor};

#[derive(
    Clone,
    Copy,
    serde::Deserialize,
    serde::Serialize,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    ts_rs::TS,
)]
#[serde(rename_all = "camelCase")]
pub enum WhisperPart {
    TinyWeights,
    TinyVad,
    SmallWeights,
    SmallVad,
    LargeV3Weights,
    LargeV3Vad,
}

const TINY: &[ModelKind] = &[
    ModelKind::WhisperPart(WhisperPart::TinyWeights),
    ModelKind::WhisperPart(WhisperPart::TinyVad),
];
const SMALL: &[ModelKind] = &[
    ModelKind::WhisperPart(WhisperPart::SmallWeights),
    ModelKind::WhisperPart(WhisperPart::SmallVad),
];
const LARGE: &[ModelKind] = &[
    ModelKind::WhisperPart(WhisperPart::LargeV3Weights),
    ModelKind::WhisperPart(WhisperPart::LargeV3Vad),
];
impl WhisperPart {
    pub(crate) const fn descriptor(self) -> ModelDescriptor {
        let (variant, sha256, download_size, file) = match self {
            Self::TinyWeights => (
                WhisperVariant::Tiny,
                "c2085835d3f50733e2ff6e4b41ae8a2b8d8110461e18821b09a15c40c42d1cca",
                43_537_433,
                "model.bin",
            ),
            Self::SmallWeights => (
                WhisperVariant::Small,
                "49c8fb02b65e6049d5fa6c04f81f53b867b5ec9540406812c643f177317f779f",
                264_464_607,
                "model.bin",
            ),
            Self::LargeV3Weights => (
                WhisperVariant::LargeV3,
                "64d182b440b98d5203c4f9bd541544d84c605196c4f7b845dfa11fb23594d1e2",
                3_095_033_483,
                "model.bin",
            ),
            Self::TinyVad | Self::SmallVad | Self::LargeV3Vad => {
                let variant = match self {
                    Self::TinyVad => WhisperVariant::Tiny,
                    Self::SmallVad => WhisperVariant::Small,
                    _ => WhisperVariant::LargeV3,
                };
                (
                    variant,
                    "2aa269b785eeb53a82983a20501ddf7c1d9c48e33ab63a41391ac6c9f7fb6987",
                    885_098,
                    "vad.bin",
                )
            }
        };
        ModelDescriptor {
            snapshot: SnapshotKind::Whisper(variant),
            file,
            sha256,
            download_size,
        }
    }
    pub(crate) fn url(self) -> String {
        let (repository, revision, file) = match self {
            Self::TinyWeights => (
                "ggerganov/whisper.cpp",
                "5359861c739e955e79d9a303bcbc70fb988958b1",
                "ggml-tiny-q8_0.bin",
            ),
            Self::SmallWeights => (
                "ggerganov/whisper.cpp",
                "5359861c739e955e79d9a303bcbc70fb988958b1",
                "ggml-small-q8_0.bin",
            ),
            Self::LargeV3Weights => (
                "ggerganov/whisper.cpp",
                "5359861c739e955e79d9a303bcbc70fb988958b1",
                "ggml-large-v3.bin",
            ),
            _ => (
                "ggml-org/whisper-vad",
                "9ffd54a1e1ee413ddf265af9913beaf518d1639b",
                "ggml-silero-v6.2.0.bin",
            ),
        };
        format!("https://huggingface.co/{repository}/resolve/{revision}/{file}")
    }
}
impl WhisperVariant {
    pub(crate) const fn models(self) -> &'static [ModelKind] {
        match self {
            Self::Tiny => TINY,
            Self::Small => SMALL,
            Self::LargeV3 => LARGE,
        }
    }
}
