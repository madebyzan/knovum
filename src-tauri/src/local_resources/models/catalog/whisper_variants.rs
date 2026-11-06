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
    BaseWeights,
    BaseVad,
    SmallWeights,
    SmallVad,
    MediumWeights,
    MediumVad,
    MediumFullWeights,
    MediumFullVad,
    LargeV3TurboQ8Weights,
    LargeV3TurboQ8Vad,
    LargeV3TurboWeights,
    LargeV3TurboVad,
    LargeV3Weights,
    LargeV3Vad,
}

const TINY: &[ModelKind] = &[
    ModelKind::WhisperPart(WhisperPart::TinyWeights),
    ModelKind::WhisperPart(WhisperPart::TinyVad),
];
const BASE: &[ModelKind] = &[
    ModelKind::WhisperPart(WhisperPart::BaseWeights),
    ModelKind::WhisperPart(WhisperPart::BaseVad),
];
const SMALL: &[ModelKind] = &[
    ModelKind::WhisperPart(WhisperPart::SmallWeights),
    ModelKind::WhisperPart(WhisperPart::SmallVad),
];
const MEDIUM: &[ModelKind] = &[
    ModelKind::WhisperPart(WhisperPart::MediumWeights),
    ModelKind::WhisperPart(WhisperPart::MediumVad),
];
const MEDIUM_FULL: &[ModelKind] = &[
    ModelKind::WhisperPart(WhisperPart::MediumFullWeights),
    ModelKind::WhisperPart(WhisperPart::MediumFullVad),
];
const LARGE_TURBO_Q8: &[ModelKind] = &[
    ModelKind::WhisperPart(WhisperPart::LargeV3TurboQ8Weights),
    ModelKind::WhisperPart(WhisperPart::LargeV3TurboQ8Vad),
];
const LARGE_TURBO: &[ModelKind] = &[
    ModelKind::WhisperPart(WhisperPart::LargeV3TurboWeights),
    ModelKind::WhisperPart(WhisperPart::LargeV3TurboVad),
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
            Self::BaseWeights => (
                WhisperVariant::Base,
                "c577b9a86e7e048a0b7eada054f4dd79a56bbfa911fbdacf900ac5b567cbb7d9",
                81_768_585,
                "model.bin",
            ),
            Self::SmallWeights => (
                WhisperVariant::Small,
                "49c8fb02b65e6049d5fa6c04f81f53b867b5ec9540406812c643f177317f779f",
                264_464_607,
                "model.bin",
            ),
            Self::MediumWeights => (
                WhisperVariant::Medium,
                "42a1ffcbe4167d224232443396968db4d02d4e8e87e213d3ee2e03095dea6502",
                823_369_779,
                "model.bin",
            ),
            Self::MediumFullWeights => (
                WhisperVariant::MediumFull,
                "6c14d5adee5f86394037b4e4e8b59f1673b6cee10e3cf0b11bbdbee79c156208",
                1_533_763_059,
                "model.bin",
            ),
            Self::LargeV3TurboQ8Weights => (
                WhisperVariant::LargeV3TurboQ8,
                "317eb69c11673c9de1e1f0d459b253999804ec71ac4c23c17ecf5fbe24e259a1",
                874_188_075,
                "model.bin",
            ),
            Self::LargeV3TurboWeights => (
                WhisperVariant::LargeV3Turbo,
                "1fc70f774d38eb169993ac391eea357ef47c88757ef72ee5943879b7e8e2bc69",
                1_624_555_275,
                "model.bin",
            ),
            Self::LargeV3Weights => (
                WhisperVariant::LargeV3,
                "64d182b440b98d5203c4f9bd541544d84c605196c4f7b845dfa11fb23594d1e2",
                3_095_033_483,
                "model.bin",
            ),
            Self::TinyVad
            | Self::BaseVad
            | Self::SmallVad
            | Self::MediumVad
            | Self::MediumFullVad
            | Self::LargeV3TurboQ8Vad
            | Self::LargeV3TurboVad
            | Self::LargeV3Vad => {
                let variant = match self {
                    Self::TinyVad => WhisperVariant::Tiny,
                    Self::BaseVad => WhisperVariant::Base,
                    Self::SmallVad => WhisperVariant::Small,
                    Self::MediumVad => WhisperVariant::Medium,
                    Self::MediumFullVad => WhisperVariant::MediumFull,
                    Self::LargeV3TurboQ8Vad => WhisperVariant::LargeV3TurboQ8,
                    Self::LargeV3TurboVad => WhisperVariant::LargeV3Turbo,
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
        const WHISPER_REPO: &str = "ggerganov/whisper.cpp";
        const WHISPER_REV: &str = "5359861c739e955e79d9a303bcbc70fb988958b1";
        let (repository, revision, file) = match self {
            Self::TinyWeights => (WHISPER_REPO, WHISPER_REV, "ggml-tiny-q8_0.bin"),
            Self::BaseWeights => (WHISPER_REPO, WHISPER_REV, "ggml-base-q8_0.bin"),
            Self::SmallWeights => (WHISPER_REPO, WHISPER_REV, "ggml-small-q8_0.bin"),
            Self::MediumWeights => (WHISPER_REPO, WHISPER_REV, "ggml-medium-q8_0.bin"),
            Self::MediumFullWeights => (WHISPER_REPO, WHISPER_REV, "ggml-medium.bin"),
            Self::LargeV3TurboQ8Weights => {
                (WHISPER_REPO, WHISPER_REV, "ggml-large-v3-turbo-q8_0.bin")
            }
            Self::LargeV3TurboWeights => (WHISPER_REPO, WHISPER_REV, "ggml-large-v3-turbo.bin"),
            Self::LargeV3Weights => (WHISPER_REPO, WHISPER_REV, "ggml-large-v3.bin"),
            _ => (
                "ggml-org/whisper-vad",
                "9ffd54a1e1ee413ddf265af9913beaf518d1639b",
                "ggml-silero-v6.2.0.bin",
            ),
        };
        crate::local_resources::models::sources::huggingface_resolve_url(repository, revision, file)
    }
}
impl WhisperVariant {
    pub(crate) const fn models(self) -> &'static [ModelKind] {
        match self {
            Self::Tiny => TINY,
            Self::Base => BASE,
            Self::Small => SMALL,
            Self::Medium => MEDIUM,
            Self::MediumFull => MEDIUM_FULL,
            Self::LargeV3TurboQ8 => LARGE_TURBO_Q8,
            Self::LargeV3Turbo => LARGE_TURBO,
            Self::LargeV3 => LARGE,
        }
    }
}
