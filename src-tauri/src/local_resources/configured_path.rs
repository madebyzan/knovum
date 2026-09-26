use std::path::PathBuf;

pub(super) fn configured_path(key: &str) -> Result<Option<PathBuf>, String> {
    validate_configured_path(key, std::env::var_os(key))
}

pub(super) fn validate_configured_path(
    key: &str,
    value: Option<std::ffi::OsString>,
) -> Result<Option<PathBuf>, String> {
    match value {
        None => Ok(None),
        Some(value) => {
            let path = PathBuf::from(value);
            if !path.is_file() {
                return Err(format!(
                    "local-resource-config-invalid: {key} must point to an existing file"
                ));
            }
            Ok(Some(path))
        }
    }
}
