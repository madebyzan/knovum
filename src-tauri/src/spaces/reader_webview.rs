use std::{
    fs,
    path::PathBuf,
    sync::{Mutex, OnceLock},
};

use serde::Deserialize;
use serde_json::Value;
use tauri::{
    http::{header, Request, Response, StatusCode},
    webview::WebviewBuilder,
    AppHandle, LogicalPosition, LogicalSize, Manager, Runtime, UriSchemeContext, Url, WebviewUrl,
};
use uuid::Uuid;

use super::assets::{access, enrichment};
use crate::desktop::{READER_PREVIEW_SCHEME, SOURCE_READER_WEBVIEW_LABEL};
use crate::foundation::app_error::AppError;

const MAIN_WINDOW_LABEL: &str = "main";
const MAX_PREVIEW_ASSET_BYTES: usize = 16 * 1024 * 1024;
const SVG_PREVIEW_CSP: &str =
    "default-src 'none'; style-src 'unsafe-inline'; img-src data: blob:; form-action 'none'";
const HTML_PREVIEW_CSP: &str =
    "sandbox allow-scripts; default-src 'none'; script-src 'unsafe-inline'; style-src 'unsafe-inline'; img-src data: blob:; font-src data:; connect-src 'none'; form-action 'none'";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AssetPreviewKind {
    Html,
    Svg,
}

struct ActivePreview {
    token: String,
    kind: AssetPreviewKind,
    bytes: Vec<u8>,
}

static ACTIVE_PREVIEW: OnceLock<Mutex<Option<ActivePreview>>> = OnceLock::new();

fn preview_slot() -> &'static Mutex<Option<ActivePreview>> {
    ACTIVE_PREVIEW.get_or_init(|| Mutex::new(None))
}

fn store_active_preview(kind: AssetPreviewKind, bytes: Vec<u8>) -> String {
    let token = Uuid::new_v4().to_string();
    if let Ok(mut guard) = preview_slot().lock() {
        *guard = Some(ActivePreview {
            token: token.clone(),
            kind,
            bytes,
        });
    }
    token
}

fn preview_csp(kind: AssetPreviewKind) -> &'static str {
    match kind {
        AssetPreviewKind::Svg => SVG_PREVIEW_CSP,
        AssetPreviewKind::Html => HTML_PREVIEW_CSP,
    }
}

fn clear_active_preview() {
    if let Ok(mut guard) = preview_slot().lock() {
        *guard = None;
    }
}

fn preview_document_url(token: &str) -> Result<Url, AppError> {
    #[cfg(target_os = "windows")]
    let raw = format!("http://{READER_PREVIEW_SCHEME}.localhost/{token}");
    #[cfg(not(target_os = "windows"))]
    let raw = format!("{READER_PREVIEW_SCHEME}://localhost/{token}");
    Url::parse(&raw).map_err(|error| AppError::new("operation_failed", error.to_string()))
}

fn empty_response(status: StatusCode) -> Response<Vec<u8>> {
    Response::builder()
        .status(status)
        .body(Vec::new())
        .unwrap_or_else(|_| Response::new(Vec::new()))
}

/// Serves Space-authorized HTML/SVG previews strictly to the isolated child reader WebView.
pub fn handle_preview_protocol<R: Runtime>(
    ctx: UriSchemeContext<'_, R>,
    request: Request<Vec<u8>>,
) -> Response<Vec<u8>> {
    if ctx.webview_label() != SOURCE_READER_WEBVIEW_LABEL {
        return empty_response(StatusCode::FORBIDDEN);
    }
    let requested = request.uri().path().trim_start_matches('/');
    let payload = preview_slot().lock().ok().and_then(|guard| {
        guard
            .as_ref()
            .filter(|active| active.token == requested)
            .map(|active| (active.kind, active.bytes.clone()))
    });
    let Some((kind, bytes)) = payload else {
        return empty_response(StatusCode::NOT_FOUND);
    };
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
        .header(header::CACHE_CONTROL, "no-store")
        .header(header::X_CONTENT_TYPE_OPTIONS, "nosniff")
        .header(header::CONTENT_SECURITY_POLICY, preview_csp(kind))
        .body(bytes)
        .unwrap_or_else(|_| empty_response(StatusCode::INTERNAL_SERVER_ERROR))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReaderWebviewBounds {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ReaderWebviewNavigation {
    Back,
    Forward,
    Reload,
}

fn parse_external_http_url(raw: &str) -> Result<Url, AppError> {
    let url = Url::parse(raw.trim())
        .map_err(|_| AppError::new("invalid_argument", "A valid HTTP(S) URL is required"))?;
    if !matches!(url.scheme(), "http" | "https")
        || !crate::desktop::navigation::is_public_reader_http_url(&url)
    {
        return Err(AppError::new(
            "invalid_argument",
            "Only public HTTP and HTTPS URLs can be opened in the embedded reader webview",
        ));
    }
    Ok(url)
}

fn resolve_preview_kind(payload: &Value) -> Result<AssetPreviewKind, AppError> {
    let kind = payload.get("kind").and_then(Value::as_str).unwrap_or("");
    if !matches!(kind, "code" | "file") {
        return Err(AppError::new(
            "invalid_argument",
            "Only HTML and SVG Space assets can be previewed in the reader webview",
        ));
    }
    let name = payload
        .get("fileName")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_ascii_lowercase();
    let lang = payload
        .get("previewLanguage")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim()
        .to_ascii_lowercase();
    if matches!(lang.as_str(), "html" | "htm" | "xhtml")
        || name.ends_with(".html")
        || name.ends_with(".htm")
        || name.ends_with(".xhtml")
    {
        return Ok(AssetPreviewKind::Html);
    }
    if lang == "svg" || name.ends_with(".svg") {
        return Ok(AssetPreviewKind::Svg);
    }
    Err(AppError::new(
        "invalid_argument",
        "Only HTML and SVG Space assets can be previewed in the reader webview",
    ))
}

fn has_html_document_tag(source: &str) -> bool {
    let head = source
        .trim_start_matches('\u{feff}')
        .trim_start()
        .get(..512)
        .unwrap_or(source)
        .to_ascii_lowercase();
    head.contains("<!doctype") || head.contains("<html")
}

fn build_preview_html(kind: AssetPreviewKind, bytes: Vec<u8>) -> Result<Vec<u8>, AppError> {
    if bytes.is_empty() || bytes.len() > MAX_PREVIEW_ASSET_BYTES {
        return Err(AppError::new(
            "invalid_argument",
            "Preview asset must be non-empty and under 16 MB",
        ));
    }
    let source = String::from_utf8(bytes)
        .map_err(|_| AppError::new("invalid_argument", "Preview asset must be valid UTF-8"))?;
    let rendered = match kind {
        AssetPreviewKind::Svg => format!(
            "<!doctype html><html><head><meta charset=\"utf-8\"><style>html,body{{margin:0;min-height:100%;display:grid;place-items:center;padding:24px;box-sizing:border-box;background:#fff}}svg{{max-width:100%;max-height:calc(100vh - 48px);height:auto}}</style></head><body>{}</body></html>",
            source.trim()
        ),
        AssetPreviewKind::Html if has_html_document_tag(&source) => source,
        AssetPreviewKind::Html => format!(
            "<!doctype html><html><head><meta charset=\"utf-8\"><style>body{{margin:24px;font-family:-apple-system,BlinkMacSystemFont,\"Segoe UI\",sans-serif;line-height:1.6;color:#1f2328;background:#fff}}</style></head><body>{source}</body></html>"
        ),
    };
    Ok(rendered.into_bytes())
}

fn validate_bounds(
    bounds: &ReaderWebviewBounds,
) -> Result<(LogicalPosition<f64>, LogicalSize<f64>), AppError> {
    if !bounds.x.is_finite()
        || !bounds.y.is_finite()
        || !bounds.width.is_finite()
        || !bounds.height.is_finite()
    {
        return Err(AppError::new(
            "invalid_argument",
            "Reader webview bounds must be finite numbers",
        ));
    }
    Ok((
        LogicalPosition::new(bounds.x.max(0.0), bounds.y.max(0.0)),
        LogicalSize::new(bounds.width.max(1.0), bounds.height.max(1.0)),
    ))
}

fn open_or_navigate_child_webview(
    app: &AppHandle,
    target: Url,
    bounds: &ReaderWebviewBounds,
) -> Result<(), AppError> {
    let (position, size) = validate_bounds(bounds)?;
    if let Some(existing) = app.get_webview(SOURCE_READER_WEBVIEW_LABEL) {
        existing
            .set_position(position)
            .map_err(|error| AppError::new("operation_failed", error.to_string()))?;
        existing
            .set_size(size)
            .map_err(|error| AppError::new("operation_failed", error.to_string()))?;
        if existing.url().ok().as_ref() != Some(&target) {
            existing
                .navigate(target)
                .map_err(|error| AppError::new("operation_failed", error.to_string()))?;
        }
        existing
            .show()
            .map_err(|error| AppError::new("operation_failed", error.to_string()))?;
        return Ok(());
    }
    let window = app
        .get_window(MAIN_WINDOW_LABEL)
        .ok_or_else(|| AppError::new("operation_failed", "Main window is unavailable"))?;
    let builder = WebviewBuilder::new(SOURCE_READER_WEBVIEW_LABEL, WebviewUrl::External(target))
        .disable_drag_drop_handler();
    window
        .add_child(builder, position, size)
        .map_err(|error| AppError::new("operation_failed", error.to_string()))?;
    Ok(())
}

#[tauri::command]
pub async fn open_reader_webview(
    app: AppHandle,
    url: String,
    bounds: ReaderWebviewBounds,
) -> Result<(), AppError> {
    let target = parse_external_http_url(&url)?;
    clear_active_preview();
    open_or_navigate_child_webview(&app, target, &bounds)
}

#[tauri::command]
pub async fn open_reader_asset_preview(
    app: AppHandle,
    space_id: String,
    node_id: String,
    bounds: ReaderWebviewBounds,
) -> Result<(), AppError> {
    let payload = enrichment::node_payload::load_node_content(&app, &space_id, &node_id).await?;
    let kind = resolve_preview_kind(&payload)?;
    let source = enrichment::node_payload::required_string(&payload, "assetPath")?;
    let read_app = app.clone();
    let raw_bytes = crate::foundation::blocking::run(move || {
        let canonical = access::authorize_spaces_asset(&read_app, &PathBuf::from(source))?;
        let metadata = fs::metadata(&canonical).map_err(|error| error.to_string())?;
        if metadata.len() as usize > MAX_PREVIEW_ASSET_BYTES {
            return Err("Preview asset exceeds the 16 MB limit".into());
        }
        fs::read(&canonical).map_err(|error| error.to_string())
    })
    .await?;
    let html_bytes = build_preview_html(kind, raw_bytes)?;
    let token = store_active_preview(kind, html_bytes);
    let target = preview_document_url(&token)?;
    open_or_navigate_child_webview(&app, target, &bounds)
}

#[tauri::command]
pub async fn update_reader_webview_bounds(
    app: AppHandle,
    bounds: ReaderWebviewBounds,
    visible: bool,
) -> Result<(), AppError> {
    let Some(webview) = app.get_webview(SOURCE_READER_WEBVIEW_LABEL) else {
        return Ok(());
    };
    if !visible {
        webview
            .hide()
            .map_err(|error| AppError::new("operation_failed", error.to_string()))?;
        return Ok(());
    }
    let (position, size) = validate_bounds(&bounds)?;
    webview
        .set_position(position)
        .map_err(|error| AppError::new("operation_failed", error.to_string()))?;
    webview
        .set_size(size)
        .map_err(|error| AppError::new("operation_failed", error.to_string()))?;
    webview
        .show()
        .map_err(|error| AppError::new("operation_failed", error.to_string()))?;
    Ok(())
}

#[tauri::command]
pub async fn navigate_reader_webview(
    app: AppHandle,
    action: ReaderWebviewNavigation,
) -> Result<(), AppError> {
    let Some(webview) = app.get_webview(SOURCE_READER_WEBVIEW_LABEL) else {
        return Ok(());
    };
    match action {
        ReaderWebviewNavigation::Back => webview
            .eval("window.history.back()")
            .map_err(|error| AppError::new("operation_failed", error.to_string()))?,
        ReaderWebviewNavigation::Forward => webview
            .eval("window.history.forward()")
            .map_err(|error| AppError::new("operation_failed", error.to_string()))?,
        ReaderWebviewNavigation::Reload => webview
            .reload()
            .map_err(|error| AppError::new("operation_failed", error.to_string()))?,
    }
    Ok(())
}

#[tauri::command]
pub async fn close_reader_webview(app: AppHandle) -> Result<(), AppError> {
    clear_active_preview();
    if let Some(webview) = app.get_webview(SOURCE_READER_WEBVIEW_LABEL) {
        webview
            .close()
            .map_err(|error| AppError::new("operation_failed", error.to_string()))?;
    }
    Ok(())
}

#[cfg(test)]
#[path = "tests/reader_webview.rs"]
mod tests;
