#[derive(Clone, Copy)]
pub(crate) struct ComputeBudget {
    pub whisper_threads: usize,
}

impl ComputeBudget {
    pub fn detect() -> Result<Self, String> {
        let logical = std::thread::available_parallelism()
            .map_err(|error| format!("failed to determine available CPU parallelism: {error}"))?
            .get();
        Ok(Self::from_logical(logical))
    }

    fn from_logical(logical: usize) -> Self {
        let interactive_reserve = if logical >= 8 { 4 } else { 2 };
        let usable = logical.saturating_sub(interactive_reserve).max(1);
        Self {
            whisper_threads: usable.min(8),
        }
    }
}

#[cfg(test)]
mod tests;
