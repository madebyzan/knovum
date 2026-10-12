use futures::StreamExt;
use url::Url;

pub(super) async fn resolve_url(value: &str) -> Result<(Url, Vec<std::net::SocketAddr>), String> {
    let url = Url::parse(value).map_err(|error| error.to_string())?;
    if url.scheme() != "https"
        || !url.username().is_empty()
        || url.password().is_some()
        || url.fragment().is_some()
    {
        return Err("package-source-requires-public-https".into());
    }
    let host = url
        .host_str()
        .ok_or("package-source-host-required")?
        .trim_matches(['[', ']']);
    let addresses = tokio::net::lookup_host((host, url.port_or_known_default().unwrap_or(443)))
        .await
        .map_err(|error| error.to_string())?;
    let addresses = addresses.collect::<Vec<_>>();
    if addresses.is_empty()
        || addresses
            .iter()
            .any(|address| !crate::foundation::public_network::public_address(address.ip()))
    {
        return Err("package-source-requires-public-address".into());
    }
    Ok((url, addresses))
}

pub(super) async fn download(value: &str, limit: usize) -> Result<Vec<u8>, String> {
    let (url, addresses) = resolve_url(value).await?;
    let host = url
        .host_str()
        .ok_or("package-source-host-required")?
        .trim_matches(['[', ']']);
    let client = crate::foundation::http::client_builder()
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .resolve_to_addrs(host, &addresses)
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .map_err(|error| error.to_string())?;
    let response = client
        .get(url)
        .send()
        .await
        .map_err(|error| error.to_string())?
        .error_for_status()
        .map_err(|error| error.to_string())?;
    if response
        .content_length()
        .is_some_and(|length| length > limit as u64)
    {
        return Err("package-download-size-limit".into());
    }
    let mut stream = response.bytes_stream();
    let mut bytes = Vec::new();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|error| error.to_string())?;
        if bytes.len() + chunk.len() > limit {
            return Err("package-download-size-limit".into());
        }
        bytes.extend_from_slice(&chunk);
    }
    Ok(bytes)
}
