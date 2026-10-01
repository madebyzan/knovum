pub fn assemble(context: &str, direction: &str) -> Result<String, String> {
    let context = context.trim();
    let direction = direction.trim();
    if context.is_empty() || direction.is_empty() {
        return Err("Video source knowledge and creative direction are required".into());
    }
    Ok(format!("Create a coherent video from the selected source knowledge and creative direction. Preserve factual claims in the sources.\n\nSOURCE KNOWLEDGE\n{context}\n\nCREATIVE DIRECTION\n{direction}"))
}
