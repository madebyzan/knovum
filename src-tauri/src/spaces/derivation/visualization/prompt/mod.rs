use serde_json::Value;

/// The prompt never claims a source the model cannot see. Attached pixels are
/// announced so the instruction can refer to them; text is quoted in full. With no
/// material at all the requirement is the whole prompt: there is nothing to stay
/// faithful to, and an explanatory framing would only steer the picture.
pub fn build(intent: &str, grounding: &[Value], attached_images: usize) -> Result<String, String> {
    if grounding.is_empty() && attached_images == 0 {
        return Ok(intent.trim().to_owned());
    }
    let mut prompt = String::from(
        "Create one clear explanatory image. Do not invent unsupported facts or labels.",
    );
    if attached_images > 0 {
        prompt.push_str(&format!(
            "\n\nATTACHED IMAGES: {attached_images}. They are the visual material for this request; keep their identity, composition and style unless the intent asks otherwise."
        ));
    }
    prompt.push_str(&format!("\n\nREQUIREMENT:\n{}", intent.trim()));
    if !grounding.is_empty() {
        prompt.push_str(&format!("\n\nKNOWLEDGE:\n{}", knowledge(grounding)?));
    }
    Ok(prompt)
}

fn knowledge(sources: &[Value]) -> Result<String, String> {
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
                .ok_or("Visualization source text is unavailable")?;
            Ok::<String, String>(format!("SOURCE {} — {}\n{}", index + 1, title, text))
        })
        .collect::<Result<Vec<_>, _>>()?
        .join("\n\n---\n\n"))
}

#[cfg(test)]
mod tests;
