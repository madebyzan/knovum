mod citations;
mod evidence;
pub(crate) mod evidence_span;
pub(crate) mod prompt;
pub(crate) mod references;

pub use citations::validate_citation_snapshots;
pub use citations::validate_conversation_citations;
pub(crate) use evidence::{cite_answer, number_passages, Reading};

pub const MAX_CITATIONS: usize = 1_000;
pub const MAX_CITATION_TITLE_CHARS: usize = 512;
pub const MAX_CITATION_QUOTE_CHARS: usize = 64_000;
