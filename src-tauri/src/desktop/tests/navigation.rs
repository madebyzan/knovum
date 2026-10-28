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
