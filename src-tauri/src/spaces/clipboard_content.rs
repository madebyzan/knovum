use clipboard_rs::{Clipboard, ClipboardContent, ClipboardContext, ContentFormat};

use crate::app_error::AppError;

#[derive(serde::Serialize)]
pub(crate) struct SpaceClipboardContent {
    html: Option<String>,
    text: Option<String>,
}

#[tauri::command]
pub(crate) async fn read_space_clipboard_content() -> Result<SpaceClipboardContent, AppError> {
    Ok((crate::blocking::run(read_content).await)?)
}

fn read_content() -> Result<SpaceClipboardContent, String> {
    let clipboard = ClipboardContext::new().map_err(|error| error.to_string())?;
    #[cfg(target_os = "macos")]
    let formats = [
        ContentFormat::Html,
        ContentFormat::Text,
        ContentFormat::Other("public.url".into()),
    ];
    #[cfg(not(target_os = "macos"))]
    let formats = [ContentFormat::Html, ContentFormat::Text];
    let contents = clipboard.get(&formats).map_err(|error| error.to_string())?;
    let mut clipboard_content = SpaceClipboardContent {
        html: None,
        text: None,
    };
    for content in contents {
        match content {
            ClipboardContent::Html(html) => clipboard_content.html = Some(html),
            ClipboardContent::Text(text) => clipboard_content.text = Some(text),
            #[cfg(target_os = "macos")]
            ClipboardContent::Other(format, bytes) if format == "public.url" => {
                let empty_text = match &clipboard_content.text {
                    Some(text) => text.trim().is_empty(),
                    None => true,
                };
                if empty_text {
                    clipboard_content.text =
                        Some(String::from_utf8(bytes).map_err(|error| error.to_string())?);
                }
            }
            _ => {}
        }
    }
    Ok(clipboard_content)
}
