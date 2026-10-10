use base64::{engine::general_purpose::STANDARD, Engine};

pub struct SourceImage {
    pub bytes: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

impl SourceImage {
    pub fn data_uri(&self) -> String {
        format!("data:image/png;base64,{}", STANDARD.encode(&self.bytes))
    }
}

pub fn from_rendered(output: crate::images::EditedImageOutput) -> Result<SourceImage, String> {
    if output.png.is_empty() || output.png.len() > 64 * 1024 * 1024 {
        return Err("Rendered image source exceeds the 64 MiB byte budget".into());
    }
    Ok(SourceImage {
        bytes: output.png,
        width: output.width,
        height: output.height,
    })
}
