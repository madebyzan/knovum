use crate::ai::contracts::AiProvider;

/// Moonshot chat runs on the shared OpenAI-compatible wire; the provider
/// carries no additional capabilities.
pub struct MoonshotProvider;

impl MoonshotProvider {
    pub fn new() -> Self {
        Self
    }
}

impl Default for MoonshotProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl AiProvider for MoonshotProvider {}
