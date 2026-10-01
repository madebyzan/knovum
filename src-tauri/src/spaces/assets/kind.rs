use serde::Serialize;

use crate::file_inspection::ResourceClassification;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, ts_rs::TS)]
#[serde(rename_all = "lowercase")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub enum AssetKind {
    Markdown,
    Code,
    File,
    Image,
    Video,
    Audio,
}

impl AssetKind {
    pub const fn from_classification(classification: ResourceClassification) -> Self {
        match classification {
            ResourceClassification::SourceCode(_) => Self::Code,
            ResourceClassification::Image => Self::Image,
            ResourceClassification::Video => Self::Video,
            ResourceClassification::Audio => Self::Audio,
            ResourceClassification::TextDocument
            | ResourceClassification::Archive
            | ResourceClassification::File => Self::File,
        }
    }

    pub fn from_node_kind(kind: &str) -> Result<Self, String> {
        match kind {
            "markdown" => Ok(Self::Markdown),
            "code" => Ok(Self::Code),
            "file" => Ok(Self::File),
            "image" => Ok(Self::Image),
            "video" => Ok(Self::Video),
            "audio" => Ok(Self::Audio),
            _ => Err(format!("Unsupported imported asset kind: {kind}")),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, ts_rs::TS)]
#[serde(rename_all = "lowercase")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub enum PreparedAssetState {
    Importing,
    Processing,
}
