use std::net::IpAddr;

pub(crate) fn same_origin(left: &reqwest::Url, right: &reqwest::Url) -> bool {
    left.origin() == right.origin()
}

// Provider/Agent output URLs carry no authority to reach the local network.
// Credentials supplied by the owning caller remain scoped to the initial origin.
pub(crate) async fn resource_response(
    mut url: reqwest::Url,
    headers: reqwest::header::HeaderMap,
) -> Result<reqwest::Response, String> {
    let authority = url.clone();
    for _ in 0..=5 {
        if url.scheme() != "https" || !url.username().is_empty() || url.password().is_some() {
            return Err("Remote resources require credential-free HTTPS URLs".into());
        }
        let host = url.host_str().ok_or("Remote resource has no host")?;
        let host = host.trim_start_matches('[').trim_end_matches(']');
        let addresses: Vec<_> =
            tokio::net::lookup_host((host, url.port_or_known_default().unwrap_or(443)))
                .await
                .map_err(|error| error.to_string())?
                .collect();
        if addresses.is_empty()
            || addresses
                .iter()
                .any(|address| !public_address(address.ip()))
        {
            return Err("Remote resources must resolve only to public addresses".into());
        }
        let client = super::http::client_builder()
            .no_proxy()
            .timeout(std::time::Duration::from_secs(120))
            .redirect(reqwest::redirect::Policy::none())
            .resolve_to_addrs(host, &addresses)
            .build()
            .map_err(|error| error.to_string())?;
        let mut request = client.get(url.clone());
        if same_origin(&authority, &url) {
            request = request.headers(headers.clone());
        }
        let response = request
            .send()
            .await
            .map_err(|error| error.without_url().to_string())?;
        if response.status().is_redirection() {
            let location = response
                .headers()
                .get(reqwest::header::LOCATION)
                .and_then(|value| value.to_str().ok())
                .ok_or("Remote resource redirect has no location")?;
            url = url.join(location).map_err(|error| error.to_string())?;
        } else {
            return response
                .error_for_status()
                .map_err(|error| error.without_url().to_string());
        }
    }
    Err("Remote resource exceeded the redirect limit".into())
}

#[cfg(test)]
mod tests;

pub(crate) fn public_address(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ip) => {
            let [first, second, third, _] = ip.octets();
            !ip.is_private()
                && !ip.is_loopback()
                && !ip.is_link_local()
                && !ip.is_unspecified()
                && !ip.is_broadcast()
                && !ip.is_multicast()
                && !ip.is_documentation()
                && first != 0
                && first < 240
                && !(first == 192 && second == 0 && third == 0)
                && !(first == 192 && second == 88 && third == 99)
                && !(first == 100 && (64..=127).contains(&second))
        }
        IpAddr::V6(ip) => ip.to_ipv4_mapped().map_or_else(
            || {
                let [first, second, ..] = ip.segments();
                (first & 0xe000) == 0x2000
                    && first != 0x2002
                    && !(first == 0x2001 && ((second < 0x0200 && second != 2) || second == 0x0db8))
                    && !(first == 0x3fff && (second & 0xf000) == 0)
            },
            |ip| public_address(IpAddr::V4(ip)),
        ),
    }
}
