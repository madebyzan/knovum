use std::io::Cursor;

use base64::{engine::general_purpose::STANDARD, Engine};
use image::{ImageReader, Limits};

pub struct SourceImage {
    pub bytes: Vec<u8>,
    pub width: u32,
    pub height: u32,
    mime_type: &'static str,
}

impl SourceImage {
    pub fn data_uri(&self) -> String {
        format!(
            "data:{};base64,{}",
            self.mime_type,
            STANDARD.encode(&self.bytes)
        )
    }
}

pub fn from_rendered(output: crate::images::EditedImageOutput) -> Result<SourceImage, String> {
    if output.png.is_empty() || output.png.len() > 10 * 1024 * 1024 {
        return Err("Image operation source exceeds the 10 MB provider limit".into());
    }
    Ok(SourceImage {
        bytes: output.png,
        width: output.width,
        height: output.height,
        mime_type: "image/png",
    })
}

pub fn from_generated(bytes: &[u8]) -> Result<SourceImage, String> {
    if bytes.len() > 10 * 1024 * 1024 {
        return Err("Generated intermediate image exceeds the 10 MB provider limit".into());
    }
    let format = image::guess_format(bytes).map_err(|_| "Generated image format is unsupported")?;
    let mime = match format {
        image::ImageFormat::Png => "image/png",
        image::ImageFormat::Jpeg => "image/jpeg",
        image::ImageFormat::WebP => "image/webp",
        _ => return Err("Generated image format is unsupported".into()),
    };
    let mut reader = ImageReader::with_format(Cursor::new(bytes), format);
    let mut limits = Limits::default();
    limits.max_image_width = Some(10_000);
    limits.max_image_height = Some(10_000);
    reader.limits(limits);
    let image = reader
        .decode()
        .map_err(|error| format!("Generated image could not be decoded: {error}"))?;
    if u64::from(image.width()) * u64::from(image.height()) > 100_000_000 {
        return Err("Generated image exceeds the pixel limit".into());
    }
    Ok(SourceImage {
        bytes: bytes.to_vec(),
        width: image.width(),
        height: image.height(),
        mime_type: mime,
    })
}
