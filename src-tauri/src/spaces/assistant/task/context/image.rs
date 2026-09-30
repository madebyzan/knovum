use crate::spaces::assistant::upload;

pub(super) struct ImageAttachmentContent {
    pub bytes: Vec<u8>,
    pub media_type: &'static str,
    pub name: String,
}

pub(super) async fn thumbnail(path: std::path::PathBuf) -> Result<String, String> {
    crate::blocking::run(move || {
        use base64::Engine;
        let (width, height) = image::image_dimensions(&path).map_err(|error| error.to_string())?;
        if u64::from(width) * u64::from(height) > 24_000_000 {
            return Err("assistant-attachment-too-large: image exceeds 24 megapixels".into());
        }
        let image = image::open(path)
            .map_err(|error| error.to_string())?
            .thumbnail(144, 144);
        let mut bytes = std::io::Cursor::new(Vec::new());
        image
            .write_to(&mut bytes, image::ImageFormat::Png)
            .map_err(|error| error.to_string())?;
        Ok(format!(
            "data:image/png;base64,{}",
            base64::engine::general_purpose::STANDARD.encode(bytes.into_inner())
        ))
    })
    .await
}

pub(crate) async fn prepare_image_input_from_file(
    path: &std::path::Path,
    target: &crate::ai::models::ExecutionTarget,
) -> Result<crate::ai::contracts::ModelContentPart, String> {
    let image = read_image_attachment(path).await?;
    crate::ai::model_turn::media::prepare_image_input(
        target,
        image.bytes,
        image.media_type,
        image.name,
    )
    .await
    .map_err(|error| error.to_string())
}

pub(super) async fn read_image_attachment(
    path: &std::path::Path,
) -> Result<ImageAttachmentContent, String> {
    let media_type = mime_guess::from_path(path)
        .first_raw()
        .ok_or("assistant-image-invalid: image media type is unavailable")?;
    let source = path.to_path_buf();
    let bytes = crate::blocking::run(move || upload::read_image(&source, media_type)).await?;
    let name = path
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or("assistant-image-invalid: filename is not Unicode")?
        .to_owned();
    Ok(ImageAttachmentContent {
        bytes,
        media_type,
        name,
    })
}
