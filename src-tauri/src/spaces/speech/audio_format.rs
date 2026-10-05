use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Serialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub enum WavPrecision {
    Pcm16,
    Pcm24,
    Float32,
}

impl WavPrecision {
    pub(super) fn codec(self) -> &'static str {
        match self {
            Self::Pcm16 => "pcm_s16le",
            Self::Pcm24 => "pcm_s24le",
            Self::Float32 => "pcm_f32le",
        }
    }
}
