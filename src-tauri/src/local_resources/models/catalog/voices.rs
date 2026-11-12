//! Voice-cloning snapshots: the audio.cpp GGUF exports of IndexTTS2 (bilibili model licence),
//! Qwen3-TTS Base (Apache-2.0) and Fun-CosyVoice3 (Apache-2.0), each beside its licence. Every
//! export embeds its configuration and tokenizer, so one weights file is the whole model.

use serde::{Deserialize, Serialize};

use super::{ModelDescriptor, ModelKind, SnapshotKind};

const GGUF_REPO: (&str, &str) = (
    "audio-cpp/audio.cpp-gguf",
    "73ce4162f6cce370dd533c6782b772135c1b2a8e",
);

#[derive(
    Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord, Hash, ts_rs::TS,
)]
#[serde(rename_all = "camelCase")]
pub enum VoiceVariant {
    IndexTts2,
    IndexTts2F16,
    Qwen3Tts,
    Qwen3TtsBf16,
    Qwen3TtsSmall,
    CosyVoice3,
}

#[derive(Clone, Copy, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord, Hash, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub enum VoiceFile {
    Weights,
    License,
}

#[derive(Clone, Copy, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord, Hash, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VoicePart {
    pub variant: VoiceVariant,
    pub file: VoiceFile,
}

/// One pinned file: its directory in the GGUF repository, its name, size and SHA-256.
struct Pin {
    directory: &'static str,
    file: &'static str,
    size: u64,
    sha256: &'static str,
}

/// A licence pinned at the commit it was read from.
struct License {
    url: &'static str,
    size: u64,
    sha256: &'static str,
}

const INDEX_TTS2_LICENSE: License = License {
    url: "https://raw.githubusercontent.com/index-tts/index-tts/d9e41aac89fd00b3d71497fddb287b7f24613712/LICENSE",
    size: 10_554,
    sha256: "53875532237e0a97b17a41721556ff57e93193fc566e48c3a0febe6e35bb5343",
};
const QWEN3_TTS_LICENSE: License = License {
    url: "https://raw.githubusercontent.com/QwenLM/Qwen3-TTS/022e286b98fbec7e1e916cb940cdf532cd9f488e/LICENSE",
    size: 11_343,
    sha256: "a44a6081c73ad75f0255bb2bb5cab74ef1829565a895a24e53a4f11290ab7655",
};
const COSYVOICE_LICENSE: License = License {
    url: "https://raw.githubusercontent.com/FunAudioLLM/CosyVoice/074ca6dc9e80a2f424f1f74b48bdd7d3fea531cc/LICENSE",
    size: 11_357,
    sha256: "c71d239df91726fc519c6eb72d318ec65820627232b2f796219e87dcf35d0ab4",
};

const fn parts(variant: VoiceVariant) -> [ModelKind; 2] {
    [
        ModelKind::VoicePart(VoicePart {
            variant,
            file: VoiceFile::Weights,
        }),
        ModelKind::VoicePart(VoicePart {
            variant,
            file: VoiceFile::License,
        }),
    ]
}

const INDEX_TTS2: [ModelKind; 2] = parts(VoiceVariant::IndexTts2);
const INDEX_TTS2_F16: [ModelKind; 2] = parts(VoiceVariant::IndexTts2F16);
const QWEN3_TTS: [ModelKind; 2] = parts(VoiceVariant::Qwen3Tts);
const QWEN3_TTS_BF16: [ModelKind; 2] = parts(VoiceVariant::Qwen3TtsBf16);
const QWEN3_TTS_SMALL: [ModelKind; 2] = parts(VoiceVariant::Qwen3TtsSmall);
const COSYVOICE3: [ModelKind; 2] = parts(VoiceVariant::CosyVoice3);

impl VoiceVariant {
    pub(crate) const fn models(self) -> &'static [ModelKind] {
        match self {
            Self::IndexTts2 => &INDEX_TTS2,
            Self::IndexTts2F16 => &INDEX_TTS2_F16,
            Self::Qwen3Tts => &QWEN3_TTS,
            Self::Qwen3TtsBf16 => &QWEN3_TTS_BF16,
            Self::Qwen3TtsSmall => &QWEN3_TTS_SMALL,
            Self::CosyVoice3 => &COSYVOICE3,
        }
    }

    /// The identity a voice revision stores and its worker is leased under.
    pub(crate) const fn spec_id(self) -> &'static str {
        match self {
            Self::IndexTts2 => "index-tts2-q8_0-v1",
            Self::IndexTts2F16 => "index-tts2-f16-v1",
            Self::Qwen3Tts => "qwen3-tts-1.7b-q8_0-v1",
            Self::Qwen3TtsBf16 => "qwen3-tts-1.7b-bf16-v1",
            Self::Qwen3TtsSmall => "qwen3-tts-0.6b-q8_0-v1",
            Self::CosyVoice3 => "cosyvoice3-0.5b-q8_0-v1",
        }
    }

    pub(crate) const fn snapshot_name(self) -> &'static str {
        match self {
            Self::IndexTts2 => "index-tts2-q8_0",
            Self::IndexTts2F16 => "index-tts2-f16",
            Self::Qwen3Tts => "qwen3-tts-1.7b-q8_0",
            Self::Qwen3TtsBf16 => "qwen3-tts-1.7b-bf16",
            Self::Qwen3TtsSmall => "qwen3-tts-0.6b-q8_0",
            Self::CosyVoice3 => "cosyvoice3-0.5b-q8_0",
        }
    }

    const fn weights(self) -> Pin {
        match self {
            Self::IndexTts2 => Pin {
                directory: "IndexTTS2-GGUF",
                file: "index-tts2-q8_0.gguf",
                size: 3_633_895_968,
                sha256: "a56e09f49fd39a894dfe1b065ab94884e9ed3515a92f75aff06fb6e6714b6b2c",
            },
            Self::IndexTts2F16 => Pin {
                directory: "IndexTTS2-GGUF",
                file: "index-tts2-f16.gguf",
                size: 4_646_905_664,
                sha256: "4bed1d526a81a7a7bfd5ebd72006bf7168ef061261b9d85b6dce30cfc882fda8",
            },
            Self::Qwen3Tts => Pin {
                directory: "Qwen3-TTS-12Hz-1.7B-Base-GGUF",
                file: "qwen3-tts-12hz-1.7b-base-q8_0_v2.gguf",
                size: 2_695_186_240,
                sha256: "92641a8022d521efd79a5db45e952589b3f2b781680f9abfcf26f075d37b86c6",
            },
            Self::Qwen3TtsBf16 => Pin {
                directory: "Qwen3-TTS-12Hz-1.7B-Base-GGUF",
                file: "qwen3-tts-12hz-1.7b-base-bf16.gguf",
                size: 4_203_169_600,
                sha256: "db237375e0f04d10fd0ac0afc2855e6844fdcb04a7d471e8a4b6c36bdb6030d7",
            },
            Self::Qwen3TtsSmall => Pin {
                directory: "Qwen3-TTS-12Hz-0.6B-Base-GGUF",
                file: "qwen3-tts-12hz-0.6b-base-q8_0.gguf",
                size: 1_991_217_728,
                sha256: "c7d0e6c9485a25ae1f6a4c0796f0023230a0051c44cbd2bd05d73770f2cce637",
            },
            Self::CosyVoice3 => Pin {
                directory: "CosyVoice3-GGUF",
                file: "cosyvoice3-q8_0.gguf",
                size: 2_257_658_336,
                sha256: "d13bd731ead92d6d514d5749d9b59ea1b3ae735569fa46978250caaf5c983133",
            },
        }
    }

    const fn license(self) -> License {
        match self {
            Self::IndexTts2 | Self::IndexTts2F16 => INDEX_TTS2_LICENSE,
            Self::Qwen3Tts | Self::Qwen3TtsBf16 | Self::Qwen3TtsSmall => QWEN3_TTS_LICENSE,
            Self::CosyVoice3 => COSYVOICE_LICENSE,
        }
    }
}

impl VoicePart {
    pub(crate) const fn descriptor(self) -> ModelDescriptor {
        let (file, sha256, download_size) = match self.file {
            VoiceFile::Weights => {
                let pin = self.variant.weights();
                (pin.file, pin.sha256, pin.size)
            }
            VoiceFile::License => {
                let license = self.variant.license();
                ("LICENSE", license.sha256, license.size)
            }
        };
        ModelDescriptor {
            snapshot: SnapshotKind::Voice(self.variant),
            file,
            sha256,
            download_size,
        }
    }

    pub(crate) fn url(self) -> String {
        use crate::local_resources::models::sources::huggingface_resolve_url;
        match self.file {
            VoiceFile::Weights => {
                let pin = self.variant.weights();
                huggingface_resolve_url(
                    GGUF_REPO.0,
                    GGUF_REPO.1,
                    &format!("{}/{}", pin.directory, pin.file),
                )
            }
            VoiceFile::License => self.variant.license().url.into(),
        }
    }
}
