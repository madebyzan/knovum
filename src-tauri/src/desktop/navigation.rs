//! The window shows the app and nothing else. A link that navigated it — an Agent's answer linking
//! a path in its working directory — replaced the whole app with a page that could not load, which
//! looked like a restart and lost everything unsaved. The renderer hands external links to the
//! system; this is the boundary that holds for every frame whatever a page does.

use tauri::{Runtime, Url, Webview};

pub const SOURCE_READER_WEBVIEW_LABEL: &str = "source-reader-web";
pub const READER_PREVIEW_SCHEME: &str = "knovum-preview";

pub(super) fn allows<R: Runtime>(webview: &Webview<R>, url: &Url) -> bool {
    if webview.label() == SOURCE_READER_WEBVIEW_LABEL {
        let allowed = allows_reader_webview(url);
        if !allowed {
            log::warn!(
                "Refused to navigate {} to unsupported {} URL",
                webview.label(),
                url.scheme()
            );
        }
        return allowed;
    }
    let dev_url = tauri::is_dev()
        .then(|| tauri::Manager::config(webview).build.dev_url.as_ref())
        .flatten();
    let allowed = serves_app(url, dev_url);
    if !allowed {
        log::warn!(
            "Refused to navigate {} away from the app to a {} URL",
            webview.label(),
            url.scheme()
        );
    }
    allowed
}

fn allows_reader_webview(url: &Url) -> bool {
    match url.scheme() {
        "about" => true,
        "http" | "https" => is_public_reader_http_url(url),
        READER_PREVIEW_SCHEME => url.host_str() == Some("localhost"),
        _ => false,
    }
}

pub(crate) fn is_public_reader_http_url(url: &Url) -> bool {
    let Some(host) = url.host_str() else {
        return false;
    };
    let lower = host.trim_end_matches('.').to_ascii_lowercase();
    if lower == "localhost" || lower.ends_with(".localhost") || lower.ends_with(".local") {
        return false;
    }
    let ip_candidate = lower.trim_start_matches('[').trim_end_matches(']');
    if let Ok(ip) = ip_candidate.parse::<std::net::IpAddr>() {
        return crate::foundation::public_network::public_address(ip);
    }
    true
}

/// What the app itself serves: its one page (routing lives in the fragment), its assets, and
/// documents it creates in place. A path link resolves against the app's own origin, so the origin
/// alone does not make a destination the app. The `http(s)://*.localhost` forms are how Windows
/// and Android address the same protocols.
fn serves_app(url: &Url, dev_url: Option<&Url>) -> bool {
    match url.scheme() {
        "about" | "blob" | "data" => true,
        "asset" => url.host_str() == Some("localhost"),
        "tauri" => url.host_str() == Some("localhost") && is_app_page(url),
        "http" | "https" => match url.host_str() {
            Some("asset.localhost") => true,
            Some("tauri.localhost") => is_app_page(url),
            _ => dev_url.is_some_and(|dev| dev.origin() == url.origin()) && is_app_page(url),
        },
        _ => false,
    }
}

fn is_app_page(url: &Url) -> bool {
    matches!(url.path(), "" | "/" | "/index.html")
}

#[cfg(test)]
#[path = "tests/navigation.rs"]
mod tests;
