use std::fs;

const MAX_THUMBNAIL_BYTES: usize = 10 * 1024 * 1024;

#[tauri::command]
pub async fn persist_canvas_thumbnail(
    app: tauri::AppHandle,
    bytes: Vec<u8>,
) -> Result<String, String> {
    crate::blocking::run(move || persist(&app, &bytes)).await
}

fn persist(app: &tauri::AppHandle, bytes: &[u8]) -> Result<String, String> {
    if bytes.len() < 4
        || bytes.len() > MAX_THUMBNAIL_BYTES
        || !bytes.starts_with(&[0xff, 0xd8])
        || !bytes.ends_with(&[0xff, 0xd9])
    {
        return Err("Canvas thumbnail must be a valid JPEG no larger than 10 MiB".into());
    }
    let target =
        super::asset_directory(app)?.join(format!("canvas-thumbnail-{}.jpg", uuid::Uuid::new_v4()));
    fs::write(&target, bytes).map_err(|error| error.to_string())?;
    Ok(target.to_string_lossy().into_owned())
}

#[cfg(test)]
mod tests;
