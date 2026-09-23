pub fn assemble(script: &str, direction: &str) -> Result<String, String> {
    let script = script.trim();
    let direction = direction.trim();
    if script.is_empty() || direction.is_empty() {
        return Err("Video script and creative direction are required".into());
    }
    Ok(format!(
        "Create a coherent video that follows this grounded script exactly. Do not introduce factual claims that are absent from it.\n\nSCRIPT\n{script}\n\nCREATIVE DIRECTION\n{direction}"
    ))
}
