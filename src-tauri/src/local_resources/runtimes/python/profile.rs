use serde::{Deserialize, Serialize};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, ts_rs::TS,
)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum RuntimeProfile {
    Agent,
}

impl RuntimeProfile {
    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::Agent => "agent",
        }
    }

    #[cfg(not(debug_assertions))]
    pub(super) fn descriptor_bytes(self) -> &'static [u8] {
        match self {
            Self::Agent => {
                include_bytes!(concat!(env!("OUT_DIR"), "/python-agent-descriptor.json"))
            }
        }
    }

    #[cfg(debug_assertions)]
    pub(super) fn lock_bytes(self) -> &'static [u8] {
        match self {
            Self::Agent => {
                include_bytes!(concat!(env!("OUT_DIR"), "/python-agent-dependencies.lock"))
            }
        }
    }

    #[cfg(debug_assertions)]
    fn producer_bytes(self) -> &'static [u8] {
        match self {
            Self::Agent => {
                include_bytes!(concat!(env!("OUT_DIR"), "/python-agent-producer.source"))
            }
        }
    }

    #[cfg(debug_assertions)]
    fn recipe_bytes(self) -> &'static [u8] {
        match self {
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
            Self::Agent => "knovum_agent_runtime/__main__.py",
        }
    }
}
