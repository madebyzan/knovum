use super::{ModelDescriptor, ModelKind, SnapshotKind};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord, Hash, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[repr(usize)]
pub enum MossNanoPart {
    Manifest,
    DecodeStep,
    GlobalWeights,
    LocalCached,
    LocalDecoder,
    LocalSampled,
    LocalWeights,
    Prefill,
    Tokenizer,
    TtsMetadata,
    CodecMetadata,
    CodecDecode,
    CodecWeights,
    CodecStep,
    CodecEncodeWeights,
    CodecEncode,
    TtsReadme,
    CodecReadme,
}

pub(crate) const MOSS_NANO_MODELS: &[ModelKind] = &[
    ModelKind::MossNanoPart(MossNanoPart::Manifest),
    ModelKind::MossNanoPart(MossNanoPart::DecodeStep),
    ModelKind::MossNanoPart(MossNanoPart::GlobalWeights),
    ModelKind::MossNanoPart(MossNanoPart::LocalCached),
    ModelKind::MossNanoPart(MossNanoPart::LocalDecoder),
    ModelKind::MossNanoPart(MossNanoPart::LocalSampled),
    ModelKind::MossNanoPart(MossNanoPart::LocalWeights),
    ModelKind::MossNanoPart(MossNanoPart::Prefill),
    ModelKind::MossNanoPart(MossNanoPart::Tokenizer),
    ModelKind::MossNanoPart(MossNanoPart::TtsMetadata),
    ModelKind::MossNanoPart(MossNanoPart::CodecMetadata),
    ModelKind::MossNanoPart(MossNanoPart::CodecDecode),
    ModelKind::MossNanoPart(MossNanoPart::CodecWeights),
    ModelKind::MossNanoPart(MossNanoPart::CodecStep),
    ModelKind::MossNanoPart(MossNanoPart::CodecEncodeWeights),
    ModelKind::MossNanoPart(MossNanoPart::CodecEncode),
    ModelKind::MossNanoPart(MossNanoPart::TtsReadme),
    ModelKind::MossNanoPart(MossNanoPart::CodecReadme),
];

// Files are independently published by the TTS and codec suppliers; one verified
// snapshot fixes their joint identity and presents a flat application-owned layout.
const FILES: &[(&str, &str, u64, bool)] = &[
    (
        "browser_poc_manifest.json",
        "097d80e993dc29f0bae427590b4f77084a161cb578b50d82c29f455d5faa9eee",
        503354,
        false,
    ),
    (
        "moss_tts_decode_step.onnx",
        "698cbc2fc1c2feca16e5895614ed52bbb32ded10f236c076f477b2e69abf32d8",
        291483,
        false,
    ),
    (
        "moss_tts_global_shared.data",
        "bce8312c3df6a44545302cae229b61054fe0672e0b252ba59cba47adeed831dc",
        440813568,
        false,
    ),
    (
        "moss_tts_local_cached_step.onnx",
        "aa9035fefc1c138a951a8bcfc0374fb03a25f1ece67f7f7f53bce349b84a1dd5",
        53685,
        false,
    ),
    (
        "moss_tts_local_decoder.onnx",
        "51aa754301b38550a5f9adda0ad93bd3dc95819afb511e6dcabf4a90b345a454",
        49231,
        false,
    ),
    (
        "moss_tts_local_fixed_sampled_frame.onnx",
        "40cdb00efc171c450cf91468e01429caa41b0252222cd308e978f58fe354afa8",
        471262,
        false,
    ),
    (
        "moss_tts_local_shared.data",
        "bae7782032c0fb12490ab42afe009f87ae6c75a0f0596fc7b5c08e4d5ee93916",
        229678080,
        false,
    ),
    (
        "moss_tts_prefill.onnx",
        "d56126dcd0574c2f15d98fc6b35eda68d0386b5bd9c5e38e28548d6f2ea8f3db",
        283305,
        false,
    ),
    (
        "tokenizer.model",
        "c353ee1479b536bf414c1b247f5542b6607fb8ae91320e5af1781fee200fddff",
        470897,
        false,
    ),
    (
        "tts_browser_onnx_meta.json",
        "3edf25232dcd0af3d061c837e9a968a39e2f8592e06777d740503c4f2244f95c",
        4487,
        false,
    ),
    (
        "codec_browser_onnx_meta.json",
        "3e291c883bb7d11ff2fe8e964e3e495519760358859f35c951254c7741592731",
        17036,
        true,
    ),
    (
        "moss_audio_tokenizer_decode_full.onnx",
        "0fbbafe3fd4afa2a019af5c5ced204af6e2d1db044fa40f021525d2aee95b4ac",
        681902,
        true,
    ),
    (
        "moss_audio_tokenizer_decode_shared.data",
        "e69d52e0f4e84ca27850557ee54face46632d3a5a16c89bd246c7c408466dcad",
        44198912,
        true,
    ),
    (
        "moss_audio_tokenizer_decode_step.onnx",
        "9527c86a29e1837edec1f74db57d5eeaadb3a715af3382703566460afed25855",
        351400,
        true,
    ),
    (
        "moss_audio_tokenizer_encode.data",
        "aa751265b2bab2887eac224484546b194875aa7494b607115439b3dc6b228a2c",
        44507136,
        true,
    ),
    (
        "moss_audio_tokenizer_encode.onnx",
        "eadea4a645abdcf98714c7aead122ee2ce7da6e080f9f80b977cd1ca8e19473a",
        815775,
        true,
    ),
    (
        "tts-README.md",
        "5147043da4b756d8599ceed41d2f4b34b0151d81be72ceca3a3db1e06b0e15da",
        6095,
        false,
    ),
    (
        "codec-README.md",
        "3c7b9e9ef1c8c5e5c829a077a1d32c1f7aea1668f2bf17ead318ff04bf1e6fe4",
        4892,
        true,
    ),
];

impl MossNanoPart {
    pub(crate) const fn descriptor(self) -> ModelDescriptor {
        let (file, sha256, download_size, _) = FILES[self as usize];
        ModelDescriptor {
            snapshot: SnapshotKind::MossNano,
            file,
            sha256,
            download_size,
        }
    }

    pub(crate) fn url(self) -> String {
        let (file, _, _, codec) = FILES[self as usize];
        let (repository, revision) = if codec {
            (
                "MOSS-Audio-Tokenizer-Nano-ONNX",
                "ceff0d0749bfb3fa2d61149794ec6feef0d1e1ae",
            )
        } else {
            (
                "MOSS-TTS-Nano-100M-ONNX",
                "f52645cb467506d8e18e746ddd59482685b74e58",
            )
        };
        let source_file = match self {
            Self::TtsReadme | Self::CodecReadme => "README.md",
            _ => file,
        };
        format!(
            "https://huggingface.co/OpenMOSS-Team/{repository}/resolve/{revision}/{source_file}"
        )
    }
}
