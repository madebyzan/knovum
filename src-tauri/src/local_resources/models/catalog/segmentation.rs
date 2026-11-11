//! Background-removal models. One row per model states everything the app knows about it: where
//! its weights come from, what its graph reads and how its output reads as coverage. A model is
//! added or retired here and nowhere else.
//!
//! Every row follows its author's reference inference. The output contract was read from each
//! graph's final operator and checked against the values it produces: BiRefNet ends in a
//! convolution (logits), BEN2 and IS-Net in a sigmoid (probabilities).

use serde::{Deserialize, Serialize};

use super::{ModelDescriptor, ModelKind};
use crate::{
    ai::contracts::image_spec::{Standardization, IMAGENET},
    local_resources::models::{snapshots::SnapshotKind, sources::HubSource},
};

#[derive(
    Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord, Hash, ts_rs::TS,
)]
#[serde(rename_all = "camelCase")]
pub enum SegmentationModel {
    BirefnetLite,
    BirefnetGeneral,
    BirefnetPortrait,
    Ben2Base,
    IsnetAnime,
}

/// How the source is placed into the graph's square input.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SquareFit {
    /// Stretched to the square, ignoring its aspect ratio.
    Stretch,
    /// Scaled to fit inside the square with its aspect ratio kept, centred on black.
    Letterbox,
}

/// How a graph's output reads as foreground coverage.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CoverageField {
    /// Unbounded logits; coverage is their sigmoid.
    Logits,
    /// Coverage itself, in `[0, 1]`.
    Probability,
}

pub(crate) struct SegmentationSpec {
    /// Directory the installed snapshot lives under.
    pub snapshot_name: &'static str,
    /// Identity a Run freezes and the worker is leased under.
    pub spec_id: &'static str,
    pub source: HubSource,
    pub file: &'static str,
    pub sha256: &'static str,
    pub bytes: u64,
    /// Edge of the square the graph reads and writes.
    pub edge: u32,
    pub fit: SquareFit,
    /// Applied to pixels scaled to `[0, 1]`.
    pub standardization: Standardization,
    pub output: CoverageField,
    /// Worker process peak for one 1024² pass on CPU, rounded up from measurement (BiRefNet Lite
    /// 5.9 GiB, BEN2 3.9 GiB, IS-Net 1.1 GiB; full-size BiRefNet exceeded a 7.6 GiB machine).
    /// Admission reserves it, so a machine that cannot hold the model refuses the Run instead of
    /// swapping.
    pub peak_memory_gib: u64,
}

/// No standardization: the graph reads pixels scaled to `[0, 1]`.
const UNIT: Standardization = Standardization {
    mean: [0.0; 3],
    std: [1.0; 3],
};

const BIREFNET_LITE: SegmentationSpec = SegmentationSpec {
    snapshot_name: "birefnet-lite",
    spec_id: "birefnet-general-lite-onnx:5600024376f572a557870a5eb0afb1e5961636bef4e1e22132025467d0f03333-v1",
    source: HubSource::HuggingFace {
        repository: "onnx-community/BiRefNet_lite-ONNX",
        revision: "de15b22ba131738a16dff04aab8bdf8dc32e3ac1",
    },
    file: "onnx/model.onnx",
    sha256: "5600024376f572a557870a5eb0afb1e5961636bef4e1e22132025467d0f03333",
    bytes: 224_005_088,
    edge: 1024,
    fit: SquareFit::Stretch,
    standardization: IMAGENET,
    output: CoverageField::Logits,
    peak_memory_gib: 7,
};

const BIREFNET_GENERAL: SegmentationSpec = SegmentationSpec {
    snapshot_name: "birefnet-general",
    spec_id:
        "birefnet-general-onnx:58f621f00f5d756097615970a88a791584600dcf7c45b18a0a6267535a1ebd3c-v1",
    source: HubSource::HuggingFace {
        repository: "onnx-community/BiRefNet-ONNX",
        revision: "534d3c82d3bb8b2f0867db6dfbc3a525b8e42f67",
    },
    file: "onnx/model.onnx",
    sha256: "58f621f00f5d756097615970a88a791584600dcf7c45b18a0a6267535a1ebd3c",
    bytes: 972_666_916,
    edge: 1024,
    fit: SquareFit::Stretch,
    standardization: IMAGENET,
    output: CoverageField::Logits,
    peak_memory_gib: 12,
};

const BIREFNET_PORTRAIT: SegmentationSpec = SegmentationSpec {
    snapshot_name: "birefnet-portrait",
    spec_id:
        "birefnet-portrait-onnx:1ba1c8ff5a7bbfadc8d8d13fb11d7be793f91f23d9d466549e37a854f6668f99-v1",
    source: HubSource::HuggingFace {
        repository: "onnx-community/BiRefNet-portrait-ONNX",
        revision: "dd7167f6a8b54ff7efc29a4c988938d79866464f",
    },
    file: "onnx/model.onnx",
    sha256: "1ba1c8ff5a7bbfadc8d8d13fb11d7be793f91f23d9d466549e37a854f6668f99",
    bytes: 972_666_916,
    edge: 1024,
    fit: SquareFit::Stretch,
    standardization: IMAGENET,
    output: CoverageField::Logits,
    peak_memory_gib: 12,
};

const BEN2_BASE: SegmentationSpec = SegmentationSpec {
    snapshot_name: "ben2-base",
    spec_id: "ben2-base-onnx:22cea62108ff53b7ccc20f7a008bf30494228d84b1687f29ecbe76936a998101-v1",
    source: HubSource::HuggingFace {
        repository: "PramaLLC/BEN2",
        revision: "e48a20765fb421d19dcdb0bf3cc61e802ca5ec8f",
    },
    file: "BEN2_Base.onnx",
    sha256: "22cea62108ff53b7ccc20f7a008bf30494228d84b1687f29ecbe76936a998101",
    bytes: 222_932_053,
    edge: 1024,
    fit: SquareFit::Stretch,
    standardization: UNIT,
    output: CoverageField::Probability,
    peak_memory_gib: 5,
};

/// SkyTNT's anime-segmentation, from its author's repository.
const ISNET_ANIME: SegmentationSpec = SegmentationSpec {
    snapshot_name: "isnet-anime",
    spec_id: "isnet-anime-onnx:f15622d853e8260172812b657053460e20806f04b9e05147d49af7bed31a6e99-v1",
    source: HubSource::HuggingFace {
        repository: "skytnt/anime-seg",
        revision: "493cb60893f47441b26ec4fb9a306bce9e342982",
    },
    file: "isnetis.onnx",
    sha256: "f15622d853e8260172812b657053460e20806f04b9e05147d49af7bed31a6e99",
    bytes: 176_069_933,
    edge: 1024,
    fit: SquareFit::Letterbox,
    standardization: UNIT,
    output: CoverageField::Probability,
    peak_memory_gib: 2,
};

impl SegmentationModel {
    pub(crate) const ALL: [Self; 5] = [
        Self::BirefnetLite,
        Self::BirefnetGeneral,
        Self::BirefnetPortrait,
        Self::Ben2Base,
        Self::IsnetAnime,
    ];

    pub(crate) const fn spec(self) -> &'static SegmentationSpec {
        match self {
            Self::BirefnetLite => &BIREFNET_LITE,
            Self::BirefnetGeneral => &BIREFNET_GENERAL,
            Self::BirefnetPortrait => &BIREFNET_PORTRAIT,
            Self::Ben2Base => &BEN2_BASE,
            Self::IsnetAnime => &ISNET_ANIME,
        }
    }

    /// The one file a segmentation snapshot holds.
    pub(crate) const fn weights(self) -> ModelKind {
        ModelKind::ImageSegmentation(self)
    }

    pub(crate) const fn models(self) -> &'static [ModelKind] {
        match self {
            Self::BirefnetLite => &[ModelKind::ImageSegmentation(Self::BirefnetLite)],
            Self::BirefnetGeneral => &[ModelKind::ImageSegmentation(Self::BirefnetGeneral)],
            Self::BirefnetPortrait => &[ModelKind::ImageSegmentation(Self::BirefnetPortrait)],
            Self::Ben2Base => &[ModelKind::ImageSegmentation(Self::Ben2Base)],
            Self::IsnetAnime => &[ModelKind::ImageSegmentation(Self::IsnetAnime)],
        }
    }

    pub(crate) const fn descriptor(self) -> ModelDescriptor {
        let spec = self.spec();
        ModelDescriptor {
            snapshot: SnapshotKind::ImageSegmentation(self),
            file: spec.file,
            sha256: spec.sha256,
            download_size: spec.bytes,
        }
    }

    pub(crate) fn from_spec_id(spec_id: &str) -> Result<Self, String> {
        Self::ALL
            .into_iter()
            .find(|model| model.spec().spec_id == spec_id)
            .ok_or_else(|| format!("unsupported_model: segmentation specification {spec_id}"))
    }
}
