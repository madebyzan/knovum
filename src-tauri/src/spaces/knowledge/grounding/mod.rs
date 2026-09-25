mod citations;
pub(crate) mod prompt;
pub(crate) mod references;

pub use citations::validate_assistant_citations;
pub use citations::validate_citation_snapshots;

pub const MAX_CITATIONS: usize = 1_000;
pub const MAX_CITATION_TITLE_CHARS: usize = 512;
pub const MAX_CITATION_QUOTE_CHARS: usize = 64_000;
