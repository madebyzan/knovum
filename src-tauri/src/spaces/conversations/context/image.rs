use crate::spaces::conversations::upload;

pub(super) struct ImageAttachmentContent {
    pub bytes: Vec<u8>,
    pub media_type: &'static str,
}

pub(super) async fn thumbnail(path: std::path::PathBuf) -> Result<String, String> {
    crate::foundation::blocking::run(move || {
        use base64::Engine;
        let (width, height) = image::image_dimensions(&path).map_err(|error| error.to_string())?;
        if u64::from(width) * u64::from(height) > 24_000_000 {
            return Err("conversation-attachment-too-large: image exceeds 24 megapixels".into());
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

pub(in crate::spaces::conversations) async fn image_input_from_file(
    path: &std::path::Path,
) -> Result<crate::agent_runtime::protocol::messages::AgentContentPart, String> {
    use base64::Engine;
    let image = read_image_attachment(path).await?;
    Ok(
        crate::agent_runtime::protocol::messages::AgentContentPart::Image {
            mime_type: image.media_type.into(),
            data: base64::engine::general_purpose::STANDARD.encode(image.bytes),
        },
    )
}

pub(super) async fn read_image_attachment(
    path: &std::path::Path,
) -> Result<ImageAttachmentContent, String> {
    let media_type = mime_guess::from_path(path)
        .first_raw()
        .ok_or("builtin-agent-image-invalid: image media type is unavailable")?;
    let source = path.to_path_buf();
    let bytes = crate::foundation::blocking::run(move || upload::read_image(&source, media_type))
        .await
        .map_err(|error| error.to_string())?;
    Ok(ImageAttachmentContent { bytes, media_type })
}
