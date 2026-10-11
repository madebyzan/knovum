use std::path::{Path, PathBuf};
pub(super) fn join(root: &Path, relative: &str) -> Result<PathBuf, String> {
    if relative.is_empty()
        || relative.contains(['\\', ':'])
        || relative
            .split('/')
            .any(|part| part.is_empty() || part == "." || part == "..")
    {
        return Err("agent-runtime-inventory-path-invalid".into());
    }
    Ok(root.join(relative))
}
