use serde_json::Value;

pub fn build(intent: &str, style: &str, sources: &[Value]) -> Result<String, String> {
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
                .ok_or("Visualization source text is unavailable")?;
            Ok::<String, String>(format!("SOURCE {} — {}\n{}", index + 1, title, text))
        })
        .collect::<Result<Vec<_>, _>>()?
        .join("\n\n---\n\n");
    Ok(format!(
        "Create one clear explanatory image based only on the supplied knowledge. The image is an illustration, not source evidence. Do not invent unsupported facts or labels.\n\nINTENT:\n{}\n\nVISUAL DIRECTION:\n{}\n\nKNOWLEDGE:\n{}",
        intent.trim(), style.trim(), context,
    ))
}
