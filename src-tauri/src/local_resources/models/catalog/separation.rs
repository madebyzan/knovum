//! HTDemucs source-separation snapshots: the audio.cpp GGUF export of Meta's Hybrid Transformer
//! Demucs (MIT), and its licence.

use serde::{Deserialize, Serialize};

use super::{ModelDescriptor, ModelKind, SnapshotKind};

const GGUF_REPO: (&str, &str) = (
    "audio-cpp/audio.cpp-gguf",
    "73ce4162f6cce370dd533c6782b772135c1b2a8e",
);
const LICENSE_URL: &str =
    "https://raw.githubusercontent.com/facebookresearch/demucs/e976d93ecc3865e5757426930257e200846a520a/LICENSE";

#[derive(
    Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord, Hash, ts_rs::TS,
)]
#[serde(rename_all = "camelCase")]
pub enum SeparationVariant {
    /// Q8_0 weights: the default, a quarter the size of F16.
    HtDemucs,
    /// F16 weights.
    HtDemucsF16,
}

#[derive(Clone, Copy, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord, Hash, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub enum SeparationFile {
    Weights,
    License,
}

#[derive(Clone, Copy, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord, Hash, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SeparationPart {
    pub variant: SeparationVariant,
    pub file: SeparationFile,
}

const fn parts(variant: SeparationVariant) -> [ModelKind; 2] {
    [
        ModelKind::SeparationPart(SeparationPart {
            variant,
            file: SeparationFile::Weights,
        }),
        ModelKind::SeparationPart(SeparationPart {
            variant,
            file: SeparationFile::License,
        }),
    ]
}

const HTDEMUCS_MODELS: [ModelKind; 2] = parts(SeparationVariant::HtDemucs);
const HTDEMUCS_F16_MODELS: [ModelKind; 2] = parts(SeparationVariant::HtDemucsF16);

impl SeparationVariant {
    pub(crate) const fn models(self) -> &'static [ModelKind] {
        match self {
            Self::HtDemucs => &HTDEMUCS_MODELS,
            Self::HtDemucsF16 => &HTDEMUCS_F16_MODELS,
        }
    }

    /// The identity a separation Run freezes and its worker is leased under.
    pub(crate) const fn spec_id(self) -> &'static str {
        match self {
            Self::HtDemucs => "htdemucs-q8_0-v1",
            Self::HtDemucsF16 => "htdemucs-f16-v1",
        }
    }

    pub(crate) const fn snapshot_name(self) -> &'static str {
        match self {
            Self::HtDemucs => "htdemucs-q8_0",
            Self::HtDemucsF16 => "htdemucs-f16",
        }
    }

    const fn weights_file(self) -> &'static str {
        match self {
            Self::HtDemucs => "htdemucs-q8_0.gguf",
            Self::HtDemucsF16 => "htdemucs-f16.gguf",
        }
    }
}

impl SeparationPart {
    pub(crate) const fn descriptor(self) -> ModelDescriptor {
        let (file, sha256, download_size) = match (self.variant, self.file) {
            (SeparationVariant::HtDemucs, SeparationFile::Weights) => (
                SeparationVariant::HtDemucs.weights_file(),
                "b0f532ac6e5f373aeb11fa0df73253251e133832d9c8b9942dc58f50bc5b4388",
                61_940_768,
            ),
            (SeparationVariant::HtDemucsF16, SeparationFile::Weights) => (
                SeparationVariant::HtDemucsF16.weights_file(),
                "f8c54ba35df95aafea7881eac214b58ec18a73cbcb6ff5a90f4b762631a286b6",
                84_059_168,
            ),
            (_, SeparationFile::License) => (
                "LICENSE",
                "cf9b17822d1fcd4ff32ccbe14183386fb3adf6f2ff92dc184130823f7fc28173",
                1_087,
            ),
        };
        ModelDescriptor {
            snapshot: SnapshotKind::Separation(self.variant),
            file,
            sha256,
            download_size,
        }
    }

    pub(crate) fn url(self) -> String {
        use crate::local_resources::models::sources::huggingface_resolve_url;
        match self.file {
            SeparationFile::Weights => huggingface_resolve_url(
                GGUF_REPO.0,
                GGUF_REPO.1,
                &format!("HTDemucs-GGUF/{}", self.variant.weights_file()),
            ),
            SeparationFile::License => LICENSE_URL.into(),
        }
    }
}
