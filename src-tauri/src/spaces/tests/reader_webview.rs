use serde_json::json;

use super::{
    build_preview_html, parse_external_http_url, preview_document_url, resolve_preview_kind,
    validate_bounds, AssetPreviewKind, ReaderWebviewBounds,
};

#[test]
fn admits_only_http_and_https_urls() {
    assert!(parse_external_http_url("https://example.com/article").is_ok());
    assert!(parse_external_http_url("http://localhost:8080/docs").is_ok());
    assert!(parse_external_http_url("file:///etc/passwd").is_err());
    assert!(parse_external_http_url("javascript:alert(1)").is_err());
    assert!(parse_external_http_url("tauri://localhost/").is_err());
}

#[test]
fn admits_only_html_and_svg_preview_nodes() {
    assert_eq!(
        resolve_preview_kind(&json!({"kind": "code", "fileName": "download.html"})).unwrap(),
        AssetPreviewKind::Html
    );
    assert_eq!(
        resolve_preview_kind(&json!({"kind": "code", "fileName": "icon.SVG"})).unwrap(),
        AssetPreviewKind::Svg
    );
    assert!(resolve_preview_kind(&json!({"kind": "code", "fileName": "main.ts"})).is_err());
    assert!(resolve_preview_kind(&json!({"kind": "note", "title": "hello.html"})).is_err());
}

#[test]
fn formats_html_fragments_and_svg_without_altering_full_documents() {
    let full = b"<!DOCTYPE html><html><body><div id='swagger-ui'></div></body></html>";
    assert_eq!(
        build_preview_html(AssetPreviewKind::Html, full.to_vec()).unwrap(),
        full
    );
    let fragment = String::from_utf8(
        build_preview_html(AssetPreviewKind::Html, b"<h1>Hi</h1>".to_vec()).unwrap(),
    )
    .unwrap();
    assert!(fragment.starts_with("<!doctype html>"));
    assert!(fragment.contains("<h1>Hi</h1>"));

    let svg = String::from_utf8(
        build_preview_html(
            AssetPreviewKind::Svg,
            b"<svg><circle r='5'/></svg>".to_vec(),
        )
        .unwrap(),
    )
    .unwrap();
    assert!(svg.contains("display:grid"));
    assert!(svg.contains("<svg><circle r='5'/></svg>"));
    assert!(preview_document_url("abc-123").is_ok());
}

#[test]
fn validates_and_clamps_bounds() {
    let valid = validate_bounds(&ReaderWebviewBounds {
        x: -5.0,
        y: 12.0,
        width: 600.0,
        height: 0.0,
    })
    .expect("finite bounds");
    assert_eq!(valid.0.x, 0.0);
    assert_eq!(valid.0.y, 12.0);
    assert_eq!(valid.1.width, 600.0);
    assert_eq!(valid.1.height, 1.0);
    assert!(validate_bounds(&ReaderWebviewBounds {
        x: f64::NAN,
        y: 0.0,
        width: 100.0,
        height: 100.0,
    })
    .is_err());
}
