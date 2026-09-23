use crate::ai::contracts::AiProvider;

pub struct OllamaProvider;

impl OllamaProvider {
    pub fn new() -> Self {
        Self
    }
}

impl Default for OllamaProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl AiProvider for OllamaProvider {}
