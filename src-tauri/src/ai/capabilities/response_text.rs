pub(super) fn strip_markdown_fences(raw: &str) -> &str {
    let trimmed = raw.trim();
    for prefix in ["```json", "```"] {
        if let Some(inside) = trimmed
            .strip_prefix(prefix)
            .and_then(|value| value.strip_suffix("```"))
        {
            return inside.trim();
        }
    }
    trimmed
}
