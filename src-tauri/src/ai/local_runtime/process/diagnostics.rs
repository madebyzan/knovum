use tokio::io::AsyncRead;

use crate::ai::contracts::diagnostics::RuntimeDiagnostic;

pub(super) async fn observe(stream: impl AsyncRead + Unpin) {
    if let Err(error) =
        crate::subprocess::diagnostic_lines::consume(stream, 16 * 1024, report).await
    {
        log::warn!("local inference diagnostic stream failed: {error}");
    }
}

fn report(line: &[u8]) {
    if let Some(diagnostic) = decode(line) {
        // Typed, bounded structural evidence only; messages, locals and source text are excluded.
        if let Ok(encoded) = serde_json::to_string(&diagnostic) {
            log::warn!("local inference diagnostic {encoded}");
        }
    }
}

fn decode(line: &[u8]) -> Option<RuntimeDiagnostic> {
    let diagnostic: RuntimeDiagnostic = serde_json::from_slice(line).ok()?;
    if diagnostic
        .request_id
        .as_ref()
        .is_some_and(|id| uuid::Uuid::parse_str(id).is_err())
        || !valid_symbol(&diagnostic.exception_type)
        || diagnostic.frames.len() > 8
        || diagnostic.frames.iter().any(|frame| {
            !valid_symbol(&frame.file) || !valid_symbol(&frame.function) || frame.line == 0
        })
    {
        return None;
    }
    Some(diagnostic)
}

fn valid_symbol(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"_.<>-".contains(&byte))
}

#[cfg(test)]
#[path = "tests/diagnostics.rs"]
mod tests;
