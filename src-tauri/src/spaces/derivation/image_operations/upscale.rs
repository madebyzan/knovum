use tokio_util::sync::CancellationToken;

use super::source::SourceImage;
use crate::ai::{
    providers::dashscope::{execute_image_operation, SourceImageOperation},
    transport::SecurityContext,
};

pub async fn execute(
    ctx: &SecurityContext,
    source: &SourceImage,
    target: u32,
    cancellation: &CancellationToken,
) -> Result<Vec<u8>, String> {
    if !matches!(target, 2048 | 4096 | 8192) {
        return Err("Upscale target must be 2048, 4096, or 8192 pixels".into());
    }
    let (mut width, mut height) = (source.width, source.height);
    let mut current = width.max(height);
    if width < 512 || height < 512 {
        return Err("Upscale source width and height must each be at least 512 pixels".into());
    }
    if current >= target {
        return Err("Upscale target must exceed the current image size".into());
    }
    let mut data_uri = source.data_uri();
    let mut bytes = Vec::new();
    while current < target {
        if width > 4096 || height > 4096 {
            return Err("Requested upscale exceeds the provider input limit".into());
        }
        let remaining = target.div_ceil(current);
        let factor = if remaining <= 4 {
            remaining.max(2)
        } else {
            (4096 / current).clamp(2, 4)
        } as u8;
        bytes = execute_image_operation(
            ctx,
            SourceImageOperation::Upscale {
                data_uri: &data_uri,
                factor,
            },
            cancellation,
        )
        .await?;
        let encoded = crate::blocking::run({
            let candidate = bytes.clone();
            move || super::source::from_generated(&candidate)
        })
        .await?;
        (width, height) = (encoded.width, encoded.height);
        let next = width.max(height);
        if next <= current {
            return Err("Upscale provider did not increase the image resolution".into());
        }
        current = next;
        data_uri = encoded.data_uri();
    }
    Ok(bytes)
}
