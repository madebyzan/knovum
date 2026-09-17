pub(crate) mod office;
pub(crate) mod pdf;
mod pdf_visuals;
pub(crate) mod text;
pub(crate) mod visual;

use serde_json::Value;

// Parsing owns admission for both direct imports and durable jobs. Acquire before reading bytes.
static PARSER: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn acquire_parse_permit(
    cancellation: &tokio_util::sync::CancellationToken,
) -> Result<std::sync::MutexGuard<'static, ()>, String> {
    let permit = PARSER
        .lock()
        .map_err(|_| "Document parser admission failed")?;
    if cancellation.is_cancelled() {
        return Err("Document processing cancelled".into());
    }
    Ok(permit)
}

pub(crate) struct DocumentSegment {
    pub(crate) kind: &'static str,
    pub(crate) text: String,
    pub(crate) location: Value,
    pub(crate) metadata: Value,
}
