use serde::{Deserialize, Serialize};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, ts_rs::TS,
)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum RuntimeProfile {
    Resources,
    LightInference,
    TorchInference,
    VoiceGeneration,
    Agent,
}

impl RuntimeProfile {
    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::Resources => "resources",
            Self::LightInference => "light-inference",
            Self::TorchInference => "torch-inference",
            Self::VoiceGeneration => "voice-generation",
            Self::Agent => "agent",
        }
    }

    pub(super) fn descriptor_bytes(self) -> &'static [u8] {
        match self {
            Self::Resources => include_bytes!(concat!(
                env!("OUT_DIR"),
                "/python-resources-descriptor.json"
            )),
            Self::LightInference => include_bytes!(concat!(
                env!("OUT_DIR"),
                "/python-light-inference-descriptor.json"
            )),
            Self::TorchInference => include_bytes!(concat!(
                env!("OUT_DIR"),
                "/python-torch-inference-descriptor.json"
            )),
            Self::VoiceGeneration => include_bytes!(concat!(
                env!("OUT_DIR"),
                "/python-voice-generation-descriptor.json"
            )),
            Self::Agent => {
                include_bytes!(concat!(env!("OUT_DIR"), "/python-agent-descriptor.json"))
            }
        }
    }

    #[cfg(debug_assertions)]
    pub(super) fn manifest_key(self) -> &'static str {
        match self {
            Self::Resources => "KNOVUM_PYTHON_RESOURCES_MANIFEST_PATH",
            Self::LightInference => "KNOVUM_PYTHON_LIGHT_INFERENCE_MANIFEST_PATH",
            Self::TorchInference => "KNOVUM_PYTHON_TORCH_INFERENCE_MANIFEST_PATH",
            Self::VoiceGeneration => "KNOVUM_PYTHON_VOICE_GENERATION_MANIFEST_PATH",
            Self::Agent => "KNOVUM_PYTHON_AGENT_MANIFEST_PATH",
        }
    }

    pub(super) fn entry(self) -> &'static str {
        match self {
            Self::Resources => "knovum_model_resources/server.py",
            Self::LightInference | Self::TorchInference | Self::VoiceGeneration => {
                "knovum_inference/server.py"
            }
            Self::Agent => "knovum_agent_runtime/__main__.py",
        }
    }
}
