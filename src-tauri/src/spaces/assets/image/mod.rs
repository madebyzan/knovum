mod clipboard;
pub(crate) mod commands;
mod editor;
mod validation;

pub(super) mod document;
pub(super) mod preview;
#[cfg(test)]
mod tests;

pub(crate) async fn rendered_preview(
    app: &tauri::AppHandle,
    data: &serde_json::Value,
) -> Result<std::path::PathBuf, String> {
    let document = document::from_payload(data)?;
    let render_app = app.clone();
    let path = crate::blocking::run(move || preview::prepare(&render_app, document)).await?;
    super::authorize_edited_image_preview(app, &path)
}
