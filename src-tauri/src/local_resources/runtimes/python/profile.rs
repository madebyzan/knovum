use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
pub(crate) enum RuntimeProfile {
    Inference,
    Assistant,
}

impl RuntimeProfile {
    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::Inference => "inference",
            Self::Assistant => "assistant",
        }
    }

    pub(super) fn descriptor_bytes(self) -> &'static [u8] {
        match self {
            Self::Inference => include_bytes!(concat!(
                env!("OUT_DIR"),
                "/python-inference-descriptor.json"
            )),
            Self::Assistant => include_bytes!(concat!(
                env!("OUT_DIR"),
                "/python-assistant-descriptor.json"
            )),
        }
    }

    #[cfg(debug_assertions)]
    pub(super) fn manifest_key(self) -> &'static str {
        match self {
            Self::Inference => "KNOVUM_PYTHON_INFERENCE_MANIFEST_PATH",
            Self::Assistant => "KNOVUM_PYTHON_ASSISTANT_MANIFEST_PATH",
        }
    }

    pub(super) fn entry(self) -> &'static str {
        match self {
            Self::Inference => "knovum_inference/server.py",
            Self::Assistant => "knovum_assistant/__main__.py",
        }
    }
}
