use std::path::Path;

use serde_json::Value;
use sqlx::SqlitePool;

use super::SourceInput;
use crate::foundation::scratch_file::ScratchFile;
use crate::spaces::db;

/// A selected image, rendered as the user currently sees it: the asset with its
/// pending edit applied, re-verified so a queued Run cannot send stale pixels.
pub struct RenderedImage {
    pub node_id: String,
    pub png: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

const ACCEPTED_MEDIA_TYPES: [&str; 4] = ["image/jpeg", "image/png", "image/webp", "image/bmp"];

/// Render every image among the selected sources, in selection order. Sources
/// that are not images are left to the text path and skipped here.
///
/// Rendering runs outside a write transaction - it is CPU work - and each file
/// is verified against its recorded revision before and after it is read.
pub async fn render_images(
    pool: &SqlitePool,
    space_id: &str,
    inputs: &[SourceInput],
    asset_directory: &Path,
    max_bytes_per_image: usize,
) -> Result<Vec<RenderedImage>, String> {
    let mut rendered = Vec::new();
    for input in inputs {
        let encoded: String = sqlx::query_scalar(
            "SELECT data FROM spaces_nodes WHERE id=$1 AND space_id=$2 AND deleted_at IS NULL",
        )
        .bind(&input.node_id)
        .bind(space_id)
        .fetch_optional(pool)
        .await
        .map_err(db::sql_error)?
        .ok_or("Selected image is unavailable")?;
        let data: Value = serde_json::from_str(&encoded).map_err(db::json_error)?;
        if data["kind"] != "image" {
            continue;
        }
        rendered.push(render(input, &data, asset_directory, max_bytes_per_image).await?);
    }
    Ok(rendered)
}

/// Hand rendered pixels to something that reads from disk - a video provider, an
/// external Agent - as files the Run owns until it is finished with them.
pub async fn write_png_files(
    rendered: &[RenderedImage],
    directory: &Path,
    name: &str,
) -> Result<(Vec<std::path::PathBuf>, Vec<ScratchFile>), String> {
    let mut paths = Vec::with_capacity(rendered.len());
    let mut files = Vec::with_capacity(rendered.len());
    for image in rendered {
        let path = directory.join(format!("{name}-{}.png", uuid::Uuid::new_v4()));
        files.push(ScratchFile::new(path.clone()));
        tokio::fs::write(&path, &image.png)
            .await
            .map_err(|error| error.to_string())?;
        paths.push(path);
    }
    Ok((paths, files))
}

async fn render(
    input: &SourceInput,
    data: &Value,
    asset_directory: &Path,
    max_bytes: usize,
) -> Result<RenderedImage, String> {
    let path = Path::new(
        data["assetPath"]
            .as_str()
            .ok_or("Selected image has no asset file")?,
    );
    if path.parent() != Some(asset_directory) {
        return Err("Selected image is outside the Space asset directory".into());
    }
    let media_type = mime_guess::from_path(path)
        .first_raw()
        .ok_or("Selected image format is unsupported")?;
    if !ACCEPTED_MEDIA_TYPES.contains(&media_type) {
        return Err("Selected image format is unsupported".into());
    }
    let revision = data["sourceRevision"]
        .as_str()
        .ok_or("Selected image has no asset revision")?;
    crate::file_inspection::verify_content_revision(path, revision)?;
    let source = path.to_owned();
    let edit = data.get("imageEdit").cloned().unwrap_or(Value::Null);
    let output = crate::foundation::blocking::run(move || {
        crate::images::render_source_png_within(&source, &edit, max_bytes)
    })
    .await?;
    crate::file_inspection::verify_content_revision(path, revision)?;
    Ok(RenderedImage {
        node_id: input.node_id.clone(),
        png: output.png,
        width: output.width,
        height: output.height,
    })
}
