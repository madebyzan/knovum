use super::{ImageModelFile as F, ImageModelPart, ImageModelVariant as V};
use crate::local_resources::models::model_archive::ModelArchive;

const SAM_ROOT: &str =
    "https://huggingface.co/yunyangx/EfficientSAM/resolve/1cf49585c39567bfc49e991ab8eb31f491ad4877";
const DEPTH_ROOT: &str = "https://huggingface.co/onnx-community/depth-anything-v2-small/resolve/4472b7362082ad9968fee890ca0f1e5aca36b93d";
const MIGAN_ROOT: &str = "https://huggingface.co/andraniksargsyan/migan/resolve/406830d0fa60666da0071c342ad2fbc8f30c5c64";

pub(super) const fn metadata(part: ImageModelPart) -> (&'static str, &'static str, u64) {
    match part.file {
        F::GeneralGraph => (
            "real_esrgan_general_x4v3.onnx",
            "cfa8edb48eab19aa29f6fab258640f081b44512f30c28a4b16a62ab1095d31f3",
            161_087,
        ),
        F::GeneralWeights => (
            "real_esrgan_general_x4v3.data",
            "512d0ec9940c2e9d85d27f2952f12a0b77b7841dc22df4ce9f3ea458bc98f37f",
            4_836_096,
        ),
        F::X4plusGraph => (
            "real_esrgan_x4plus.onnx",
            "29ffd5bc0277b19536cd39b737627fb2d79df9999b8329741b558498dd5e31f7",
            3_158_313,
        ),
        F::X4plusWeights => (
            "real_esrgan_x4plus.data",
            "28fada125730d3c87d504d48dd8332837f65811ec431a570ea4919ba3eee3287",
            66_737_664,
        ),
        F::MiganWeights => (
            "migan_pipeline_v2.onnx",
            "6f1f3530a1a2324b19752018ce756088b07973cda8d7d890034ace5c8a48c40b",
            28_079_181,
        ),
        F::TiEncoder => (
            "efficientsam_ti_encoder.onnx",
            "84ed466ffcc5c1f8d08409bc34a23bb364ab2c15e402cb12d4335a42be0e0951",
            24_799_761,
        ),
        F::TiDecoder => (
            "efficientsam_ti_decoder.onnx",
            "a62f8fa5ea080447c0689418d69e58f1e83e0b7adf9c142e2bd9bcc8045c0b11",
            16_565_728,
        ),
        F::SEncoder => (
            "efficientsam_s_encoder.onnx",
            "3eea4544db4647570ca5bea8ab6bbe057f2fc767ad089c14b8f67f482a2493ed",
            89_558_337,
        ),
        F::SDecoder => (
            "efficientsam_s_decoder.onnx",
            "d40a537f617c8ced3b2034b6971bc16cc291c49d3d80f740a394cdb78f0e1d06",
            16_565_728,
        ),
        F::DepthUint8Weights => (
            "model_uint8.onnx",
            "fcf51f1b230362b28690bb9d1809bf0431f29cad20534e3f589bd7285547f20d",
            27_258_801,
        ),
        F::DepthFp32Weights => (
            "model.onnx",
            "afb6a5c28f3b6bf1618c6e43f02073ef9dfdc70e937502d51603e57b0a1df10c",
            99_060_839,
        ),
        F::DepthProcessor => (
            "preprocessor_config.json",
            "03576db3c13dd0471fdf5f5e1428befcb95de063fe699879150b293dc9e0a2c6",
            461,
        ),
        F::DepthModelCard => (
            "MODEL_CARD.md",
            "2fdd0db4eb5d7725fe0f1522e03a63f0b3c7dc93c805669f3322f665fb3b4c86",
            1_578,
        ),
        F::License => match part.variant {
            V::RealEsrganGeneralX4v3 | V::RealEsrganX4plus => (
                "LICENSE",
                "4a699ec4863d96a91fc265948a0c90033f7e8735d515524dcf3444736406e0c2",
                1_519,
            ),
            V::Migan => (
                "LICENSE",
                "13348eaaebc0b07b0faf3a92db04e0858737713a917d064b599c87cac05b571f",
                1_082,
            ),
            V::EfficientSamTi
            | V::EfficientSamS
            | V::DepthAnythingV2SmallUint8
            | V::DepthAnythingV2Small => (
                "LICENSE",
                "c71d239df91726fc519c6eb72d318ec65820627232b2f796219e87dcf35d0ab4",
                11_357,
            ),
        },
    }
}

pub(super) fn url(part: ImageModelPart) -> String {
    match part.file {
        F::GeneralGraph | F::GeneralWeights | F::X4plusGraph | F::X4plusWeights => unreachable!("These files belong to the frozen release archive"),
        F::TiEncoder | F::TiDecoder | F::SEncoder | F::SDecoder => format!("{SAM_ROOT}/{}", metadata(part).0),
        F::MiganWeights => format!("{MIGAN_ROOT}/{}", metadata(part).0),
        F::DepthUint8Weights | F::DepthFp32Weights => format!("{DEPTH_ROOT}/onnx/{}", metadata(part).0),
        F::DepthProcessor => format!("{DEPTH_ROOT}/preprocessor_config.json"),
        F::DepthModelCard => format!("{DEPTH_ROOT}/README.md"),
        F::License => match part.variant {
            V::RealEsrganGeneralX4v3 | V::RealEsrganX4plus => "https://raw.githubusercontent.com/xinntao/Real-ESRGAN/a4abfb2979a7bbff3f69f58f58ae324608821e27/LICENSE".into(),
            V::Migan => format!("{MIGAN_ROOT}/LICENSE"),
            V::EfficientSamTi | V::EfficientSamS => "https://raw.githubusercontent.com/yformer/EfficientSAM/d525f622e6f640acf5a0fc37c7ca1f243da5bde0/LICENSE".into(),
            V::DepthAnythingV2SmallUint8 | V::DepthAnythingV2Small => "https://raw.githubusercontent.com/DepthAnything/Depth-Anything-V2/a561b849ebae10a6f5ef49e26c83cbbcd36c71bf/LICENSE".into(),
        },
    }
}

pub(super) const fn archive(variant: V) -> Option<ModelArchive> {
    Some(match variant {
        V::RealEsrganGeneralX4v3 => ModelArchive {
            url: "https://qaihub-public-assets.s3.us-west-2.amazonaws.com/qai-hub-models/models/real_esrgan_general_x4v3/releases/v0.62.2/real_esrgan_general_x4v3-onnx-float.zip",
            sha256: "19e208e88e906097a9df3fc5ca2318cb745c26a556348dc48fce562f36c49d38",
            bytes: 4_540_894,
            directory: "real_esrgan_general_x4v3-onnx-float",
            members: &[F::GeneralGraph, F::GeneralWeights],
        },
        V::RealEsrganX4plus => ModelArchive {
            url: "https://qaihub-public-assets.s3.us-west-2.amazonaws.com/qai-hub-models/models/real_esrgan_x4plus/releases/v0.62.2/real_esrgan_x4plus-onnx-float.zip",
            sha256: "e6cb215390f3800b56baa0e9140907d81f5143a4b96c184a635c79dfee2e28df",
            bytes: 62_174_026,
            directory: "real_esrgan_x4plus-onnx-float",
            members: &[F::X4plusGraph, F::X4plusWeights],
        },
        _ => return None,
    })
}
