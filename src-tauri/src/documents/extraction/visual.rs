use serde_json::Value;

/// A logical occurrence, not a standalone canvas node. Binary data never crosses IPC.
pub(crate) struct DocumentImage {
    pub(crate) id: String,
    pub(crate) location: Value,
    pub(crate) context: String,
    pub(crate) alt_text: Option<String>,
    pub(crate) data: Option<Vec<u8>>,
}
