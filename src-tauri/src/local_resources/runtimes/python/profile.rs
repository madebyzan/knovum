use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
pub(crate) enum RuntimeProfile {
    Inference,
    Agent,
}

impl RuntimeProfile {
    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::Inference => "inference",
            Self::Agent => "agent",
        }
    }

    pub(super) fn descriptor_bytes(self) -> &'static [u8] {
        match self {
            Self::Inference => include_bytes!(concat!(
                env!("OUT_DIR"),
                "/python-inference-descriptor.json"
            )),
            Self::Agent => {
                include_bytes!(concat!(env!("OUT_DIR"), "/python-agent-descriptor.json"))
            }
        }
    }

    #[cfg(debug_assertions)]
    pub(super) fn manifest_key(self) -> &'static str {
        match self {
            Self::Inference => "KNOVUM_PYTHON_INFERENCE_MANIFEST_PATH",
            Self::Agent => "KNOVUM_PYTHON_AGENT_MANIFEST_PATH",
        }
    }

    pub(super) fn entry(self) -> &'static str {
        match self {
            Self::Inference => "knovum_inference/server.py",
            Self::Agent => "knovum_agent_runtime/__main__.py",
        }
    }
}
