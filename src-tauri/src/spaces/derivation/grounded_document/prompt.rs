use serde_json::Value;

use crate::ai::tasks::completion::TaskInput;

const GROUNDED: &str = "Create a useful Markdown document only from the supplied Knovum knowledge sources. Cite supported factual claims inline with source IDs such as [C1] or [C1][C3]. Never invent a source ID. Preserve disagreements and uncertainty. Return only the document Markdown.";

/// With nothing selected there is no source to cite, so the document is asked for on its purpose
/// alone and claims no citations.
const UNGROUNDED: &str = "Create a useful Markdown document for the stated purpose. Do not cite sources or invent references. Return only the document Markdown.";

pub fn request(purpose: &str, sources: &[Value]) -> Result<TaskInput, String> {
    let (system_prompt, text) = if sources.is_empty() {
        (UNGROUNDED, String::new())
    } else {
        (GROUNDED, format!("SOURCES:\n{}", context(sources)?))
    };
    Ok(TaskInput {
        purpose: crate::ai::usage::UsagePurpose::Knowledge,
        system_prompt: system_prompt.into(),
        instruction: format!("Document purpose:\n{}", purpose.trim()),
        text,
        max_tokens: None,
        image: None,
    })
}

fn context(sources: &[Value]) -> Result<String, String> {
    Ok(sources
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
        .join("\n\n---\n\n"))
}
