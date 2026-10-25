/// With no selected source the creative direction is the whole prompt.
pub fn assemble(context: &str, direction: &str) -> Result<String, String> {
    let context = context.trim();
    let direction = direction.trim();
    if direction.is_empty() {
        return Err("Video creative direction is required".into());
    }
    if context.is_empty() {
        return Ok(direction.to_owned());
    }
    Ok(format!("Create a coherent video from the selected source knowledge and creative direction. Preserve factual claims in the sources.\n\nSOURCE KNOWLEDGE\n{context}\n\nCREATIVE DIRECTION\n{direction}"))
}
