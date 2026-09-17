use super::ModelDescriptor;

const DEFAULT_ENDPOINTS: [&str; 2] = ["https://huggingface.co", "https://hf-mirror.com"];

pub(super) fn download_urls(descriptor: ModelDescriptor) -> Result<Vec<reqwest::Url>, String> {
    let endpoint = std::env::var("KNOVUM_MODEL_ENDPOINT").or_else(|error| match error {
        std::env::VarError::NotPresent => std::env::var("HF_ENDPOINT"),
        error => Err(error),
    });
    match endpoint {
        Ok(endpoint) => urls(descriptor, &[&endpoint]),
        Err(std::env::VarError::NotPresent) => urls(descriptor, &DEFAULT_ENDPOINTS),
        Err(_) => Err("Model endpoint must be valid Unicode".to_string()),
    }
}

pub(super) fn urls(
    descriptor: ModelDescriptor,
    endpoints: &[&str],
) -> Result<Vec<reqwest::Url>, String> {
    endpoints
        .iter()
        .map(|endpoint| {
            let mut url = reqwest::Url::parse(endpoint)
                .map_err(|_| "Model endpoint must be an absolute HTTPS URL".to_string())?;
            if url.scheme() != "https"
                || url.host_str().is_none()
                || !url.username().is_empty()
                || url.password().is_some()
                || url.query().is_some()
                || url.fragment().is_some()
            {
                return Err(
                    "Model endpoint requires HTTPS without credentials, query or fragment"
                        .to_string(),
                );
            }
            let path = format!(
                "{}/{}/resolve/{}/{}",
                url.path().trim_end_matches('/'),
                descriptor.repository,
                descriptor.revision,
                descriptor.file
            );
            url.set_path(&path);
            url.set_query(Some("download=true"));
            Ok(url)
        })
        .collect()
}
