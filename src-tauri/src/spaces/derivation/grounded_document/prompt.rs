use serde_json::Value;

use crate::ai::models::{CompletionRequest, ExecutionTarget};

const SYSTEM: &str = "Create a useful Markdown document only from the supplied Knovum knowledge sources. Cite supported factual claims inline with source IDs such as [C1] or [C1][C3]. Never invent a source ID. Preserve disagreements and uncertainty. Return only the document Markdown.";

pub fn request(
    target: &ExecutionTarget,
    purpose: &str,
    sources: &[Value],
) -> Result<CompletionRequest, String> {
    let context = sources
        .iter()
        .enumerate()
        .map(|(index, source)| {
            let title = source
                .get("nodeTitle")
                .and_then(Value::as_str)
                .unwrap_or("Untitled");
            let text = source
                .get("text")
                .and_then(Value::as_str)
                .ok_or("Document source text is unavailable")?;
            Ok::<String, String>(format!("C{} | {}\n{}", index + 1, title, text))
        })
        .collect::<Result<Vec<_>, String>>()?
        .join("\n\n---\n\n");
    Ok(CompletionRequest {
        target: target.clone(),
        system_prompt: SYSTEM.into(),
        instruction: format!("Document purpose:\n{}", purpose.trim()),
        text: format!("SOURCES:\n{context}"),
        max_tokens: None,
    })
}
