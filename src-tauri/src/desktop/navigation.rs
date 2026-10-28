//! The window shows the app and nothing else. A link that navigated it — an Agent's answer linking
//! a path in its working directory — replaced the whole app with a page that could not load, which
//! looked like a restart and lost everything unsaved. The renderer hands external links to the
//! system; this is the boundary that holds for every frame whatever a page does.

use tauri::{Runtime, Url, Webview};

pub(super) fn allows<R: Runtime>(webview: &Webview<R>, url: &Url) -> bool {
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
    matches!(url.path(), "/" | "/index.html")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn url(value: &str) -> Url {
        Url::parse(value).unwrap()
    }

    #[test]
    fn the_app_and_what_it_serves_stay_in_the_window() {
        for allowed in [
            "tauri://localhost/",
            "tauri://localhost/index.html#/spaces/b9e3a6dc",
            "http://tauri.localhost/index.html",
            "https://tauri.localhost/",
            "asset://localhost/%2FUsers%2Fme%2Fpreview.html",
            "http://asset.localhost/preview.html",
            "about:blank",
            "about:srcdoc",
            "blob:tauri://localhost/0b6c",
            "data:text/html,<p>x</p>",
        ] {
            assert!(serves_app(&url(allowed), None), "{allowed}");
        }
    }

    #[test]
    fn a_link_never_takes_the_window_away_from_the_app() {
        // An Agent's answer linked `/Users/…/PPT摘要.md`; the page resolved it against its own
        // origin and the window left the app for a page that does not exist.
        for refused in [
            "tauri://localhost/Users/me/agent-workspaces/PPT%E6%91%98%E8%A6%81.md",
            "http://tauri.localhost/Users/me/notes.md",
            "file:///Users/me/agent-workspaces/PPT%E6%91%98%E8%A6%81.md",
            "https://example.com/",
            "http://localhost:1420/",
            "mailto:someone@example.com",
            "tauri://elsewhere/",
        ] {
            assert!(!serves_app(&url(refused), None), "{refused}");
        }
    }

    #[test]
    fn a_development_build_serves_the_app_from_its_dev_server() {
        let dev = url("http://localhost:1420");
        assert!(serves_app(
            &url("http://localhost:1420/#/space"),
            Some(&dev)
        ));
        assert!(!serves_app(
            &url("http://localhost:1420/Users/me/notes.md"),
            Some(&dev)
        ));
        assert!(!serves_app(&url("http://localhost:3000/"), Some(&dev)));
    }
}
