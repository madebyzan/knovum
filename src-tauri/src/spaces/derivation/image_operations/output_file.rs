use std::path::PathBuf;

use tauri::AppHandle;
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

pub async fn write(app: &AppHandle, png: &[u8]) -> Result<PathBuf, String> {
    let path = crate::spaces::assets::asset_directory(app)?
        .join(format!("image-operation-{}.png", Uuid::new_v4()));
    let mut file = tokio::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .await
        .map_err(|error| error.to_string())?;
    let written = async {
        file.write_all(png).await?;
        file.flush().await
    }
    .await;
    drop(file);
    if let Err(error) = written {
        let _ = tokio::fs::remove_file(&path).await;
        return Err(error.to_string());
    }
    Ok(path)
}
