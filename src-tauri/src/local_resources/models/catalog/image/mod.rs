mod files;
use super::{ModelDescriptor, ModelKind, SnapshotKind};
use serde::{Deserialize, Serialize};

pub use knovum_inference_contracts::image_compute::ImageModelVariant;

pub(crate) trait ImageModelCatalog {
    fn models(self) -> &'static [ModelKind];
    fn archive(self) -> Option<super::super::model_archive::ModelArchive>;
    fn spec_id(self) -> String;
}

#[derive(
    Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord, Hash, ts_rs::TS,
)]
#[serde(rename_all = "camelCase")]
pub enum ImageModelFile {
    GeneralGraph,
    GeneralWeights,
    X4plusGraph,
    X4plusWeights,
    LamaWeights,
    MobileSamEncoder,
    MobileSamDecoder,
    Sam2Encoder,
    Sam2Decoder,
    DepthUint8Weights,
    DepthFp32Weights,
    DepthProcessor,
    DepthModelCard,
    TextDetector,
    TextRecognizer,
    TextDictionary,
    License,
}

#[derive(
    Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord, Hash, ts_rs::TS,
)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ImageModelPart {
    pub variant: ImageModelVariant,
    pub file: ImageModelFile,
}

impl ImageModelCatalog for ImageModelVariant {
    fn models(self) -> &'static [ModelKind] {
        match self {
            Self::RealEsrganGeneralX4v3 => &[
                ModelKind::ImagePart(ImageModelPart {
                    variant: Self::RealEsrganGeneralX4v3,
                    file: ImageModelFile::GeneralGraph,
                }),
                ModelKind::ImagePart(ImageModelPart {
                    variant: Self::RealEsrganGeneralX4v3,
                    file: ImageModelFile::GeneralWeights,
                }),
                ModelKind::ImagePart(ImageModelPart {
                    variant: Self::RealEsrganGeneralX4v3,
                    file: ImageModelFile::License,
                }),
            ],
            Self::RealEsrganX4plus => &[
                ModelKind::ImagePart(ImageModelPart {
                    variant: Self::RealEsrganX4plus,
                    file: ImageModelFile::X4plusGraph,
                }),
                ModelKind::ImagePart(ImageModelPart {
                    variant: Self::RealEsrganX4plus,
                    file: ImageModelFile::X4plusWeights,
                }),
                ModelKind::ImagePart(ImageModelPart {
                    variant: Self::RealEsrganX4plus,
                    file: ImageModelFile::License,
                }),
            ],
            Self::Lama => &[
                ModelKind::ImagePart(ImageModelPart {
                    variant: Self::Lama,
                    file: ImageModelFile::LamaWeights,
                }),
                ModelKind::ImagePart(ImageModelPart {
                    variant: Self::Lama,
                    file: ImageModelFile::License,
                }),
            ],
            Self::MobileSam => &[
                ModelKind::ImagePart(ImageModelPart {
                    variant: Self::MobileSam,
                    file: ImageModelFile::MobileSamEncoder,
                }),
                ModelKind::ImagePart(ImageModelPart {
                    variant: Self::MobileSam,
                    file: ImageModelFile::MobileSamDecoder,
                }),
                ModelKind::ImagePart(ImageModelPart {
                    variant: Self::MobileSam,
                    file: ImageModelFile::License,
                }),
            ],
            Self::Sam2HieraTiny => &[
                ModelKind::ImagePart(ImageModelPart {
                    variant: Self::Sam2HieraTiny,
                    file: ImageModelFile::Sam2Encoder,
                }),
                ModelKind::ImagePart(ImageModelPart {
                    variant: Self::Sam2HieraTiny,
                    file: ImageModelFile::Sam2Decoder,
                }),
                ModelKind::ImagePart(ImageModelPart {
                    variant: Self::Sam2HieraTiny,
                    file: ImageModelFile::License,
                }),
            ],
            Self::DepthAnythingV2SmallUint8 => &[
                ModelKind::ImagePart(ImageModelPart {
                    variant: Self::DepthAnythingV2SmallUint8,
                    file: ImageModelFile::DepthUint8Weights,
                }),
                ModelKind::ImagePart(ImageModelPart {
                    variant: Self::DepthAnythingV2SmallUint8,
                    file: ImageModelFile::DepthProcessor,
                }),
                ModelKind::ImagePart(ImageModelPart {
                    variant: Self::DepthAnythingV2SmallUint8,
                    file: ImageModelFile::DepthModelCard,
                }),
                ModelKind::ImagePart(ImageModelPart {
                    variant: Self::DepthAnythingV2SmallUint8,
                    file: ImageModelFile::License,
                }),
            ],
            Self::DepthAnythingV2Small => &[
                ModelKind::ImagePart(ImageModelPart {
                    variant: Self::DepthAnythingV2Small,
                    file: ImageModelFile::DepthFp32Weights,
                }),
                ModelKind::ImagePart(ImageModelPart {
                    variant: Self::DepthAnythingV2Small,
                    file: ImageModelFile::DepthProcessor,
                }),
                ModelKind::ImagePart(ImageModelPart {
                    variant: Self::DepthAnythingV2Small,
                    file: ImageModelFile::DepthModelCard,
                }),
                ModelKind::ImagePart(ImageModelPart {
                    variant: Self::DepthAnythingV2Small,
                    file: ImageModelFile::License,
                }),
            ],
            Self::PpOcrV6Small => &[
                ModelKind::ImagePart(ImageModelPart {
                    variant: Self::PpOcrV6Small,
                    file: ImageModelFile::TextDetector,
                }),
                ModelKind::ImagePart(ImageModelPart {
                    variant: Self::PpOcrV6Small,
                    file: ImageModelFile::TextRecognizer,
                }),
                ModelKind::ImagePart(ImageModelPart {
                    variant: Self::PpOcrV6Small,
                    file: ImageModelFile::TextDictionary,
                }),
                ModelKind::ImagePart(ImageModelPart {
                    variant: Self::PpOcrV6Small,
                    file: ImageModelFile::License,
                }),
            ],
        }
    }
    fn archive(self) -> Option<super::super::model_archive::ModelArchive> {
        files::archive(self)
    }
    fn spec_id(self) -> String {
        format!(
            "{}-cpu-v1:{}",
            self.name(),
            SnapshotKind::Image(self).digest()
        )
    }
}

impl ImageModelPart {
    pub(crate) const fn descriptor(self) -> ModelDescriptor {
        let (file, sha256, download_size) = files::metadata(self);
        ModelDescriptor {
            snapshot: SnapshotKind::Image(self.variant),
            file,
            sha256,
            download_size,
        }
    }
    pub(crate) fn url(self) -> String {
        files::url(self)
    }
}
