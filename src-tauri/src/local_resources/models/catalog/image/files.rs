use super::{ImageModelFile as F, ImageModelPart, ImageModelVariant as V};
use crate::local_resources::models::{
    model_archive::ModelArchive, sources::huggingface_resolve_url,
};

const DEPTH_REPO: (&str, &str) = (
    "onnx-community/depth-anything-v2-small",
    "4472b7362082ad9968fee890ca0f1e5aca36b93d",
);
/// Carve's ONNX export of the authors' Big LaMa; its README states the export is the original
/// model's, so the license is the authors'.
const LAMA_REPO: (&str, &str) = (
    "Carve/LaMa-ONNX",
    "c3c0c9e468934d62e79c329e35d82dd09ff8c444",
);
const PADDLE_OCR_DET_REPO: (&str, &str) = (
    "PaddlePaddle/PP-OCRv6_small_det_onnx",
    "28fe5895c24fd108c19eb3e8479f4ab385fbfc62",
);
const PADDLE_OCR_REC_REPO: (&str, &str) = (
    "PaddlePaddle/PP-OCRv6_small_rec_onnx",
    "b8f84f0b80c529de40b4fbb3544b84fa7233a513",
);
const PADDLE_OCR_ROOT: &str =
    "https://raw.githubusercontent.com/PaddlePaddle/PaddleOCR/b03f46425e8ff4442b268ce449e3eef758146cd4";

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
        F::LamaWeights => (
            "lama_fp32.onnx",
            "1faef5301d78db7dda502fe59966957ec4b79dd64e16f03ed96913c7a4eb68d6",
            208_044_816,
        ),
        F::MobileSamEncoder => (
            "mobile_sam.encoder.onnx",
            "20deef402855b31222b528f52b04807e41ebe47216ac0e39a0729f43491a0209",
            28_157_093,
        ),
        F::MobileSamDecoder => (
            "sam_vit_h_4b8939.decoder.onnx",
            "22cf85e35d14182f4b4712364264c06b22edbef63f065189586f080ef4e2f325",
            16_500_272,
        ),
        F::Sam2Encoder => (
            "sam2.1_hiera_tiny.encoder.onnx",
            "667384d1e686de6828b841ac8a24db0fafa2b3452494225f82eeedac56141230",
            109_471_931,
        ),
        F::Sam2Decoder => (
            "sam2.1_hiera_tiny.decoder.onnx",
            "c40f5aa7d37b681cd500481a85d44839fd81c93dce1e86271a2c866470d22105",
            16_519_561,
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
        F::TextDetector => (
            "pp_ocrv6_small_det.onnx",
            "d73e0058b7a8086bbd57f3d10b8bcd4ff95363f67e06e2762b5e814fe9c9410e",
            9_880_512,
        ),
        F::TextRecognizer => (
            "pp_ocrv6_small_rec.onnx",
            "5435fd747c9e0efe15a96d0b378d5bd157e9492ed8fd80edf08f30d02fa24634",
            21_159_378,
        ),
        F::TextDictionary => (
            "ppocrv6_dict.txt",
            "b5f2bfe2bdd9448429e3e82b51c789775d9b42f2403d082b00662eb77e401c5d",
            74_947,
        ),
        F::License => match part.variant {
            V::PpOcrV6Small => (
                "LICENSE",
                "3840c5c0c61c294264d2dd77b8777be6ddd90121ef4e0e64abcd22edea581d6e",
                11_376,
            ),
            V::RealEsrganGeneralX4v3 | V::RealEsrganX4plus => (
                "LICENSE",
                "4a699ec4863d96a91fc265948a0c90033f7e8735d515524dcf3444736406e0c2",
                1_519,
            ),
            V::Lama => (
                "LICENSE",
                "4ceeeac5a802e86c413c22b16cce8e9a22027b0250c97e6f8ac97c14cf0542c0",
                11_348,
            ),
            V::MobileSam
            | V::Sam2HieraTiny
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
        F::GeneralGraph
        | F::GeneralWeights
        | F::X4plusGraph
        | F::X4plusWeights
        | F::MobileSamEncoder
        | F::MobileSamDecoder
        | F::Sam2Encoder
        | F::Sam2Decoder => {
            unreachable!("These files belong to the frozen release archive")
        }
        F::LamaWeights => huggingface_resolve_url(LAMA_REPO.0, LAMA_REPO.1, metadata(part).0),
        F::DepthUint8Weights | F::DepthFp32Weights => {
            huggingface_resolve_url(DEPTH_REPO.0, DEPTH_REPO.1, &format!("onnx/{}", metadata(part).0))
        }
        F::DepthProcessor => huggingface_resolve_url(DEPTH_REPO.0, DEPTH_REPO.1, "preprocessor_config.json"),
        F::DepthModelCard => huggingface_resolve_url(DEPTH_REPO.0, DEPTH_REPO.1, "README.md"),
        F::TextDetector => {
            huggingface_resolve_url(PADDLE_OCR_DET_REPO.0, PADDLE_OCR_DET_REPO.1, "inference.onnx")
        }
        F::TextRecognizer => {
            huggingface_resolve_url(PADDLE_OCR_REC_REPO.0, PADDLE_OCR_REC_REPO.1, "inference.onnx")
        }
        F::TextDictionary => format!("{PADDLE_OCR_ROOT}/ppocr/utils/dict/ppocrv6_dict.txt"),
        F::License => match part.variant {
            V::PpOcrV6Small => format!("{PADDLE_OCR_ROOT}/LICENSE"),
            V::RealEsrganGeneralX4v3 | V::RealEsrganX4plus => {
                "https://raw.githubusercontent.com/xinntao/Real-ESRGAN/a4abfb2979a7bbff3f69f58f58ae324608821e27/LICENSE".into()
            }
            V::Lama => {
                "https://raw.githubusercontent.com/advimman/lama/786f5936b27fb3dacd2b1ad799e4de968ea697e7/LICENSE".into()
            }
            V::MobileSam => {
                "https://raw.githubusercontent.com/ChaoningZhang/MobileSAM/f706ad9c4eb7f219c00d9050e46328518ffb65d2/LICENSE".into()
            }
            V::Sam2HieraTiny => {
                "https://raw.githubusercontent.com/facebookresearch/sam2/2b90b9f5ceec907a1c18123530e92e794ad901a4/LICENSE".into()
            }
            V::DepthAnythingV2SmallUint8 | V::DepthAnythingV2Small => {
                "https://raw.githubusercontent.com/DepthAnything/Depth-Anything-V2/a561b849ebae10a6f5ef49e26c83cbbcd36c71bf/LICENSE".into()
            }
        },
    }
}

pub(super) const fn archive(variant: V) -> Option<ModelArchive> {
    Some(match variant {
        V::RealEsrganGeneralX4v3 => ModelArchive {
            url: "https://qaihub-public-assets.s3.us-west-2.amazonaws.com/qai-hub-models/models/real_esrgan_general_x4v3/releases/v0.62.2/real_esrgan_general_x4v3-onnx-float.zip",
            sha256: "19e208e88e906097a9df3fc5ca2318cb745c26a556348dc48fce562f36c49d38",
            bytes: 4_540_894,
            directory: Some("real_esrgan_general_x4v3-onnx-float"),
            members: &[F::GeneralGraph, F::GeneralWeights],
        },
        V::RealEsrganX4plus => ModelArchive {
            url: "https://qaihub-public-assets.s3.us-west-2.amazonaws.com/qai-hub-models/models/real_esrgan_x4plus/releases/v0.62.2/real_esrgan_x4plus-onnx-float.zip",
            sha256: "e6cb215390f3800b56baa0e9140907d81f5143a4b96c184a635c79dfee2e28df",
            bytes: 62_174_026,
            directory: Some("real_esrgan_x4plus-onnx-float"),
            members: &[F::X4plusGraph, F::X4plusWeights],
        },
        V::MobileSam => ModelArchive {
            url: "https://huggingface.co/vietanhdev/segment-anything-onnx-models/resolve/9effc01a9e135621d710d49159f1ffb0b6f724dc/mobile_sam_20230629.zip",
            sha256: "41aff2660b7531becfee21fb257c49933ddc892c554507bdb775bf504d443942",
            bytes: 36_655_105,
            directory: None,
            members: &[F::MobileSamEncoder, F::MobileSamDecoder],
        },
        V::Sam2HieraTiny => ModelArchive {
            url: "https://huggingface.co/vietanhdev/segment-anything-2.1-onnx-models/resolve/6a3ac868340a3196a349050a6efae22a5acc0330/sam2.1_hiera_tiny_20260221.zip",
            sha256: "c602cb3f6fd297312a415f885f0df3f0eef9fbb334b069c9d30e828f2ae7c69a",
            bytes: 116_507_723,
            directory: None,
            members: &[F::Sam2Encoder, F::Sam2Decoder],
        },
        _ => return None,
    })
}
