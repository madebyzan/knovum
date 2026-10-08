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
    FasterTinyConfig,
    FasterTinyWeights,
    FasterTinyTokenizer,
    FasterTinyVocabulary,
    FasterLargeV3Config,
    FasterLargeV3Weights,
    FasterLargeV3Tokenizer,
    FasterLargeV3Vocabulary,
    MlxTinyConfig,
    MlxTinyWeights,
    MlxLargeV3Config,
    MlxLargeV3Weights,
}

const FASTERTINY_PARTS: &[ModelKind] = &[
    ModelKind::WhisperPart(WhisperPart::FasterTinyConfig),
    ModelKind::WhisperPart(WhisperPart::FasterTinyWeights),
    ModelKind::WhisperPart(WhisperPart::FasterTinyTokenizer),
    ModelKind::WhisperPart(WhisperPart::FasterTinyVocabulary),
];
const FASTERLARGEV3_PARTS: &[ModelKind] = &[
    ModelKind::WhisperPart(WhisperPart::FasterLargeV3Config),
    ModelKind::WhisperPart(WhisperPart::FasterLargeV3Weights),
    ModelKind::WhisperPart(WhisperPart::FasterLargeV3Tokenizer),
    ModelKind::WhisperPart(WhisperPart::FasterLargeV3Vocabulary),
];
const MLXTINY_PARTS: &[ModelKind] = &[
    ModelKind::WhisperPart(WhisperPart::MlxTinyConfig),
    ModelKind::WhisperPart(WhisperPart::MlxTinyWeights),
];
const MLXLARGEV3_PARTS: &[ModelKind] = &[
    ModelKind::WhisperPart(WhisperPart::MlxLargeV3Config),
    ModelKind::WhisperPart(WhisperPart::MlxLargeV3Weights),
];
impl WhisperPart {
    pub(crate) const fn descriptor(self) -> ModelDescriptor {
        match self {
            Self::FasterTinyConfig => ModelDescriptor {
                snapshot: SnapshotKind::Whisper(WhisperVariant::FasterTiny),
                file: "config.json",

                sha256: "a73a28cdfe1c43ccc7202fa333d1f89c202477271407ae9a7f19afa52039cac8",
                download_size: 2249,
            },
            Self::FasterTinyWeights => ModelDescriptor {
                snapshot: SnapshotKind::Whisper(WhisperVariant::FasterTiny),
                file: "model.bin",

                sha256: "dcb76c6586fc06cbdac6dd21f14cfd129cc4cdd9dce19bf4ffa62e59cbe6e6d1",
                download_size: 75538270,
            },
            Self::FasterTinyTokenizer => ModelDescriptor {
                snapshot: SnapshotKind::Whisper(WhisperVariant::FasterTiny),
                file: "tokenizer.json",

                sha256: "fb7b63191e9bb045082c79fd742a3106a12c99513ab30df4a0d47fa6cb6fd0ab",
                download_size: 2203239,
            },
            Self::FasterTinyVocabulary => ModelDescriptor {
                snapshot: SnapshotKind::Whisper(WhisperVariant::FasterTiny),
                file: "vocabulary.txt",

                sha256: "34ce3fe1c5041027b3f8d42912270993f986dbc4bb34cf27f951e34a1e453913",
                download_size: 459861,
            },
            Self::FasterLargeV3Config => ModelDescriptor {
                snapshot: SnapshotKind::Whisper(WhisperVariant::FasterLargeV3),
                file: "config.json",

                sha256: "a9306624f5ec14270a014b647e5c316b6e03a662c369758d1b90697a7b0655b9",
                download_size: 2394,
            },
            Self::FasterLargeV3Weights => ModelDescriptor {
                snapshot: SnapshotKind::Whisper(WhisperVariant::FasterLargeV3),
                file: "model.bin",

                sha256: "69f74147e3334731bc3a76048724833325d2ec74642fb52620eda87352e3d4f1",
                download_size: 3087284237,
            },
            Self::FasterLargeV3Tokenizer => ModelDescriptor {
                snapshot: SnapshotKind::Whisper(WhisperVariant::FasterLargeV3),
                file: "tokenizer.json",

                sha256: "6d8cbd7cd0d8d5815e478dac67b85a26bbe77c1f5e0c6d76d1ce2abc0e5f21ca",
                download_size: 2480617,
            },
            Self::FasterLargeV3Vocabulary => ModelDescriptor {
                snapshot: SnapshotKind::Whisper(WhisperVariant::FasterLargeV3),
                file: "vocabulary.json",

                sha256: "c69260f2ab26d659b7c398f9a2b2b48ed0df16c3b47d7326782fd9cba71690c1",
                download_size: 1068114,
            },
            Self::MlxTinyConfig => ModelDescriptor {
                snapshot: SnapshotKind::Whisper(WhisperVariant::MlxTiny),
                file: "config.json",

                sha256: "aaff20ce8f69beddee3fe0cc1e08f4e92f58586cb9f12ba00a6f73cbfec1cb1c",
                download_size: 262,
            },
            Self::MlxTinyWeights => ModelDescriptor {
                snapshot: SnapshotKind::Whisper(WhisperVariant::MlxTiny),
                file: "weights.npz",

                sha256: "0e03a5993d6eea43b07ee2dcc772b0e4cef5bb227257dacc24bf289387d49186",
                download_size: 74418540,
            },
            Self::MlxLargeV3Config => ModelDescriptor {
                snapshot: SnapshotKind::Whisper(WhisperVariant::MlxLargeV3),
                file: "config.json",

                sha256: "34982ce6ae286095000f82ae9583b3431639e8b092bf60c961f203745e6500e3",
                download_size: 269,
            },
            Self::MlxLargeV3Weights => ModelDescriptor {
                snapshot: SnapshotKind::Whisper(WhisperVariant::MlxLargeV3),
                file: "weights.npz",

                sha256: "05ff791ce3630fae47e7c51004e9666204d786246ec07cac6110af768099b40d",
                download_size: 3083520416,
            },
        }
    }
}
impl WhisperVariant {
    pub(crate) const fn models(self) -> &'static [ModelKind] {
        match self {
            Self::FasterTiny => FASTERTINY_PARTS,
            Self::FasterLargeV3 => FASTERLARGEV3_PARTS,
            Self::MlxTiny => MLXTINY_PARTS,
            Self::MlxLargeV3 => MLXLARGEV3_PARTS,
        }
    }
    pub(crate) fn source(self) -> crate::local_resources::models::sources::HubSource {
        let (repository, revision) = match self {
            Self::FasterTiny => (
                "Systran/faster-whisper-tiny",
                "d90ca5fe260221311c53c58e660288d3deb8d356",
            ),
            Self::FasterLargeV3 => (
                "Systran/faster-whisper-large-v3",
                "edaa852ec7e145841d8ffdb056a99866b5f0a478",
            ),
            Self::MlxTiny => (
                "mlx-community/whisper-tiny-mlx",
                "6caf9c55601caafbe6508a8b0d216bdf4783c4e8",
            ),
            Self::MlxLargeV3 => (
                "mlx-community/whisper-large-v3-mlx",
                "49e6aa286ad60c14352c404340ded53710378a11",
            ),
        };
        crate::local_resources::models::sources::HubSource::HuggingFace {
            repository,
            revision,
            endpoint: "https://huggingface.co",
        }
    }
}
