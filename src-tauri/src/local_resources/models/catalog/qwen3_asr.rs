//! Qwen3-ASR transcription snapshots: the audio.cpp GGUF export of Qwen3-ASR-1.7B (Apache-2.0),
//! its licence and model card, beside the whisper.cpp Silero VAD binary that selects its speech
//! regions. The export embeds its configuration and tokenizer, so one weights file is the model.

use serde::{Deserialize, Serialize};

use super::{ModelDescriptor, ModelKind, SnapshotKind};

const GGUF_REPO: (&str, &str) = (
    "audio-cpp/audio.cpp-gguf",
    "73ce4162f6cce370dd533c6782b772135c1b2a8e",
);
const VAD_REPO: (&str, &str) = (
    "ggml-org/whisper-vad",
    "9ffd54a1e1ee413ddf265af9913beaf518d1639b",
);
const LICENSE_URL: &str =
    "https://raw.githubusercontent.com/QwenLM/Qwen3-ASR/7c6daf77a2421100f5fb066495372c00129d39ff/LICENSE";
const README_REPO: (&str, &str) = (
    "Qwen/Qwen3-ASR-1.7B-hf",
    "bcd2b5b7f32b480ab5790554cfa8347f246a14f3",
);

#[derive(
    Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord, Hash, ts_rs::TS,
)]
#[serde(rename_all = "camelCase")]
pub enum Qwen3AsrVariant {
    /// Q8_0 weights: the default, well under half the size of F16.
    Qwen3Asr,
    /// F16 weights.
    Qwen3AsrF16,
}

#[derive(Clone, Copy, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord, Hash, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub enum Qwen3AsrFile {
    Weights,
    Vad,
    License,
    Readme,
}

#[derive(Clone, Copy, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord, Hash, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Qwen3AsrPart {
    pub variant: Qwen3AsrVariant,
    pub file: Qwen3AsrFile,
}

const fn parts(variant: Qwen3AsrVariant) -> [ModelKind; 4] {
    [
        ModelKind::Qwen3AsrPart(Qwen3AsrPart {
            variant,
            file: Qwen3AsrFile::Weights,
        }),
        ModelKind::Qwen3AsrPart(Qwen3AsrPart {
            variant,
            file: Qwen3AsrFile::Vad,
        }),
        ModelKind::Qwen3AsrPart(Qwen3AsrPart {
            variant,
            file: Qwen3AsrFile::License,
        }),
        ModelKind::Qwen3AsrPart(Qwen3AsrPart {
            variant,
            file: Qwen3AsrFile::Readme,
        }),
    ]
}

const QWEN3_ASR_MODELS: [ModelKind; 4] = parts(Qwen3AsrVariant::Qwen3Asr);
const QWEN3_ASR_F16_MODELS: [ModelKind; 4] = parts(Qwen3AsrVariant::Qwen3AsrF16);

impl Qwen3AsrVariant {
    pub(crate) const fn models(self) -> &'static [ModelKind] {
        match self {
            Self::Qwen3Asr => &QWEN3_ASR_MODELS,
            Self::Qwen3AsrF16 => &QWEN3_ASR_F16_MODELS,
        }
    }

    /// The identity a transcription Run freezes and its worker is leased under.
    pub(crate) const fn spec_id(self) -> &'static str {
        match self {
            Self::Qwen3Asr => "qwen3-asr-1.7b-q8_0-v1",
            Self::Qwen3AsrF16 => "qwen3-asr-1.7b-f16-v1",
        }
    }

    pub(crate) const fn snapshot_name(self) -> &'static str {
        match self {
            Self::Qwen3Asr => "qwen3-asr-1.7b-q8_0",
            Self::Qwen3AsrF16 => "qwen3-asr-1.7b-f16",
        }
    }

    pub(crate) const fn compute_type(self) -> &'static str {
        match self {
            Self::Qwen3Asr => "q8_0",
            Self::Qwen3AsrF16 => "f16",
        }
    }

    const fn weights_file(self) -> &'static str {
        match self {
            Self::Qwen3Asr => "qwen3-asr-1.7b-q8_0.gguf",
            Self::Qwen3AsrF16 => "qwen3-asr-1.7b-f16.gguf",
        }
    }
}

impl Qwen3AsrPart {
    pub(crate) const fn descriptor(self) -> ModelDescriptor {
        let (file, sha256, download_size) = match (self.variant, self.file) {
            (Qwen3AsrVariant::Qwen3Asr, Qwen3AsrFile::Weights) => (
                "model.gguf",
                "3bc2462eca3a3a8f56abcd860d0943ffdc0546547dde55a9d08589607d6f3936",
                2_473_017_600,
            ),
            (Qwen3AsrVariant::Qwen3AsrF16, Qwen3AsrFile::Weights) => (
                "model.gguf",
                "59baf62f2454a1b4d1d9ffed730dbf1d0bce28ed6fec30897de17f9d6d452fc3",
                4_087_660_800,
            ),
            (_, Qwen3AsrFile::Vad) => (
                "vad.bin",
                "2aa269b785eeb53a82983a20501ddf7c1d9c48e33ab63a41391ac6c9f7fb6987",
                885_098,
            ),
            (_, Qwen3AsrFile::License) => (
                "LICENSE",
                "a44a6081c73ad75f0255bb2bb5cab74ef1829565a895a24e53a4f11290ab7655",
                11_343,
            ),
            (_, Qwen3AsrFile::Readme) => (
                "README.md",
                "631fbed15be1ee46f9e82569b5c62906ae2af40daef879bd5a475d1b8833248f",
                16_538,
            ),
        };
        ModelDescriptor {
            snapshot: SnapshotKind::Qwen3Asr(self.variant),
            file,
            sha256,
            download_size,
        }
    }

    pub(crate) fn url(self) -> String {
        use crate::local_resources::models::sources::huggingface_resolve_url;
        match self.file {
            Qwen3AsrFile::Weights => huggingface_resolve_url(
                GGUF_REPO.0,
                GGUF_REPO.1,
                &format!("Qwen3-ASR-1.7B-GGUF/{}", self.variant.weights_file()),
            ),
            Qwen3AsrFile::Vad => {
                huggingface_resolve_url(VAD_REPO.0, VAD_REPO.1, "ggml-silero-v6.2.0.bin")
            }
            Qwen3AsrFile::License => LICENSE_URL.into(),
            Qwen3AsrFile::Readme => {
                huggingface_resolve_url(README_REPO.0, README_REPO.1, "README.md")
            }
        }
    }
}
