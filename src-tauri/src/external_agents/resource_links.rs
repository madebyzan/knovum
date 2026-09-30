use std::{
    path::{Path, PathBuf},
    time::Duration,
};

use agent_client_protocol::{schema::v1::ResourceLink, Error};
use sha2::{Digest, Sha256};
use tokio::io::AsyncWriteExt;

pub(super) async fn materialize(
    workspace: &Path,
    link: ResourceLink,
    output_identity: Option<&str>,
) -> Result<Option<PathBuf>, Error> {
    acquire(workspace, link, output_identity)
        .await
        .map_err(|error| Error::invalid_request().data(error))
}

async fn acquire(
    workspace: &Path,
    link: ResourceLink,
    output_identity: Option<&str>,
) -> Result<Option<PathBuf>, String> {
    let url = tauri::Url::parse(&link.uri).map_err(|error| error.to_string())?;
    if url.scheme() == "file" {
        let path = url.to_file_path().map_err(|()| "Invalid Agent file URL")?;
        let metadata = tokio::fs::symlink_metadata(&path)
            .await
            .map_err(|error| error.to_string())?;
        let canonical = tokio::fs::canonicalize(&path)
            .await
            .map_err(|error| error.to_string())?;
        let root = tokio::fs::canonicalize(workspace)
            .await
            .map_err(|error| error.to_string())?;
        if !metadata.is_file() || metadata.file_type().is_symlink() || !canonical.starts_with(&root)
        {
            return Err("Agent output file must be a regular file inside its workspace".into());
        }
        return Ok(Some(canonical));
    }
    require_https(&url)?;
    let mut response = remote_response(url.clone()).await?;
    let extension = remote_extension(
        response.url(),
        response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok()),
        link.mime_type.as_deref(),
    )?;
    let identity = Sha256::digest(url.as_str().as_bytes());
    let base = output_identity
        .map(str::to_owned)
        .unwrap_or_else(|| format!("resource-{identity:x}"));
    let pending = workspace.join(format!(".{base}-{}.partial", uuid::Uuid::new_v4()));
    let mut file = tokio::fs::File::create(&pending)
        .await
        .map_err(|error| error.to_string())?;
    let mut content_identity = Sha256::new();
    let result = async {
        while let Some(chunk) = response
            .chunk()
            .await
            .map_err(|error| error.without_url().to_string())?
        {
            content_identity.update(&chunk);
            file.write_all(&chunk)
                .await
                .map_err(|error| error.to_string())?;
        }
        file.sync_all().await.map_err(|error| error.to_string())?;
        drop(file);
        let content_identity = content_identity.finalize();
        let target = workspace.join(format!("{base}-{content_identity:x}.{extension}"));
        tokio::fs::rename(&pending, &target)
            .await
            .map_err(|error| error.to_string())?;
        Ok(target)
    }
    .await;
    if result.is_err() {
        let _ = tokio::fs::remove_file(&pending).await;
    }
    result.map(Some)
}

fn remote_extension(
    final_url: &tauri::Url,
    response_content_type: Option<&str>,
    declared_mime_type: Option<&str>,
) -> Result<String, String> {
    let response_extensions = response_content_type
        .and_then(|value| value.parse::<mime::Mime>().ok())
        .and_then(|value| mime_guess::get_mime_extensions_str(value.essence_str()));
    Path::new(final_url.path())
        .extension()
        .and_then(|value| value.to_str())
        .filter(|value| value.len() <= 10 && value.chars().all(|c| c.is_ascii_alphanumeric()))
        .filter(|value| response_extensions.is_none_or(|extensions| extensions.contains(value)))
        .map(str::to_owned)
        .or_else(|| {
            response_extensions.and_then(|values| values.first().map(|value| (*value).into()))
        })
        .or_else(|| {
            declared_mime_type
                .and_then(|value| value.parse::<mime::Mime>().ok())
                .and_then(|value| mime_guess::get_mime_extensions_str(value.essence_str()))
                .and_then(|values| values.first().map(|value| (*value).into()))
        })
        .ok_or_else(|| "Agent resource link has no file type".into())
}

async fn remote_response(mut url: tauri::Url) -> Result<reqwest::Response, String> {
    for _ in 0..=5 {
        require_https(&url)?;
        let host = url.host_str().ok_or("Agent resource has no host")?;
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
            return Err("Agent remote output must resolve only to public addresses".into());
        }
        let client = reqwest::Client::builder()
            .no_proxy()
            .timeout(Duration::from_secs(120))
            .redirect(reqwest::redirect::Policy::none())
            .resolve_to_addrs(host, &addresses)
            .build()
            .map_err(|error| error.to_string())?;
        let response = client
            .get(url.clone())
            .send()
            .await
            .map_err(|error| error.without_url().to_string())?;
        if response.status().is_redirection() {
            let location = response
                .headers()
                .get(reqwest::header::LOCATION)
                .and_then(|value| value.to_str().ok())
                .ok_or("Agent output redirect has no location")?;
            url = url.join(location).map_err(|error| error.to_string())?;
        } else {
            return response
                .error_for_status()
                .map_err(|error| error.without_url().to_string());
        }
    }
    Err("Agent output exceeded redirect limit".into())
}

fn public_address(ip: std::net::IpAddr) -> bool {
    match ip {
        std::net::IpAddr::V4(ip) => {
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
                && !(first == 198 && (18..=19).contains(&second))
        }
        std::net::IpAddr::V6(ip) => ip.to_ipv4_mapped().map_or_else(
            || {
                (ip.segments()[0] & 0xe000) == 0x2000
                    && ip.segments()[0] != 0x2002
                    && !ip.is_loopback()
                    && !ip.is_unspecified()
                    && !ip.is_multicast()
                    && !ip.is_unique_local()
                    && !ip.is_unicast_link_local()
                    && !(ip.segments()[0] == 0x2001 && ip.segments()[1] == 0x0db8)
            },
            |ip| public_address(std::net::IpAddr::V4(ip)),
        ),
    }
}

fn require_https(url: &tauri::Url) -> Result<(), String> {
    if url.scheme() != "https"
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
    {
        return Err("Agent remote output must use credential-free HTTPS".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn workspace_links_do_not_copy_and_external_files_are_rejected() {
        let root = std::env::temp_dir().join(format!("agent-links-{}", uuid::Uuid::new_v4()));
        tokio::fs::create_dir(&root).await.unwrap();
        let file = root.join("report.md");
        tokio::fs::write(&file, "# Report").await.unwrap();
        let uri = tauri::Url::from_file_path(&file).unwrap();
        acquire(&root, ResourceLink::new("Report", uri.to_string()), None)
            .await
            .unwrap();
        assert_eq!(
            tokio::fs::read_dir(&root)
                .await
                .unwrap()
                .next_entry()
                .await
                .unwrap()
                .unwrap()
                .path(),
            file
        );
        let outside = root.with_extension("txt");
        tokio::fs::write(&outside, "private").await.unwrap();
        let uri = tauri::Url::from_file_path(&outside).unwrap();
        assert!(
            acquire(&root, ResourceLink::new("Outside", uri.to_string()), None)
                .await
                .is_err()
        );
        assert!(require_https(&tauri::Url::parse("http://example.com/a.png").unwrap()).is_err());
        tokio::fs::remove_dir_all(root).await.unwrap();
        tokio::fs::remove_file(outside).await.unwrap();
    }
    #[test]
    fn remote_resources_reject_private_and_special_addresses() {
        for address in [
            "127.0.0.1",
            "10.0.0.1",
            "172.16.0.1",
            "192.168.0.1",
            "169.254.169.254",
            "100.64.0.1",
            "::1",
            "fc00::1",
            "fe80::1",
            "::ffff:127.0.0.1",
            "fec0::1",
        ] {
            assert!(!public_address(address.parse().unwrap()), "{address}");
        }
        assert!(public_address("8.8.8.8".parse().unwrap()));
        assert!(public_address("2606:4700:4700::1111".parse().unwrap()));
    }

    #[test]
    fn remote_type_uses_final_url_and_response_content_type() {
        assert_eq!(
            remote_extension(
                &tauri::Url::parse("https://cdn.example.com/final.png").unwrap(),
                Some("image/png"),
                Some("application/octet-stream"),
            )
            .unwrap(),
            "png"
        );
        let jpeg_extension = remote_extension(
            &tauri::Url::parse("https://cdn.example.com/download.php").unwrap(),
            Some("image/jpeg; charset=binary"),
            Some("application/octet-stream"),
        )
        .unwrap();
        assert!(mime_guess::get_mime_extensions_str("image/jpeg")
            .unwrap()
            .contains(&jpeg_extension.as_str()));
    }
}
