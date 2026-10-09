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

    #[cfg(not(debug_assertions))]
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
    pub(super) fn lock_bytes(self) -> &'static [u8] {
        match self {
            Self::Resources => include_bytes!(concat!(
                env!("OUT_DIR"),
                "/python-resources-dependencies.lock"
            )),
            Self::LightInference => include_bytes!(concat!(
                env!("OUT_DIR"),
                "/python-light-inference-dependencies.lock"
            )),
            Self::TorchInference => include_bytes!(concat!(
                env!("OUT_DIR"),
                "/python-torch-inference-dependencies.lock"
            )),
            Self::VoiceGeneration => include_bytes!(concat!(
                env!("OUT_DIR"),
                "/python-voice-generation-dependencies.lock"
            )),
            Self::Agent => {
                include_bytes!(concat!(env!("OUT_DIR"), "/python-agent-dependencies.lock"))
            }
        }
    }

    #[cfg(debug_assertions)]
    fn producer_bytes(self) -> &'static [u8] {
        match self {
            Self::Resources => include_bytes!(concat!(
                env!("OUT_DIR"),
                "/python-resources-producer.source"
            )),
            Self::LightInference => include_bytes!(concat!(
                env!("OUT_DIR"),
                "/python-light-inference-producer.source"
            )),
            Self::TorchInference => include_bytes!(concat!(
                env!("OUT_DIR"),
                "/python-torch-inference-producer.source"
            )),
            Self::VoiceGeneration => include_bytes!(concat!(
                env!("OUT_DIR"),
                "/python-voice-generation-producer.source"
            )),
            Self::Agent => {
                include_bytes!(concat!(env!("OUT_DIR"), "/python-agent-producer.source"))
            }
        }
    }

    #[cfg(debug_assertions)]
    fn recipe_bytes(self) -> &'static [u8] {
        match self {
            Self::Resources => {
                include_bytes!(concat!(env!("OUT_DIR"), "/python-resources-recipe.json"))
            }
            Self::LightInference => include_bytes!(concat!(
                env!("OUT_DIR"),
                "/python-light-inference-recipe.json"
            )),
            Self::TorchInference => include_bytes!(concat!(
                env!("OUT_DIR"),
                "/python-torch-inference-recipe.json"
            )),
            Self::VoiceGeneration => include_bytes!(concat!(
                env!("OUT_DIR"),
                "/python-voice-generation-recipe.json"
            )),
            Self::Agent => include_bytes!(concat!(env!("OUT_DIR"), "/python-agent-recipe.json")),
        }
    }

    #[cfg(debug_assertions)]
    pub(super) fn development_manifest(self) -> std::path::PathBuf {
        use sha2::{Digest, Sha256};
        let identity = Sha256::new()
            .chain_update(self.recipe_bytes())
            .chain_update(self.producer_bytes())
            .chain_update(self.lock_bytes())
            .finalize();
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("Desktop crate belongs to the repository")
            .join(".runtime-development")
            .join(self.name())
            .join(crate::local_resources::tools::platform_directory())
            .join(hex::encode(identity))
            .join("runtime")
            .join(super::MANIFEST_NAME)
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
