use tokio_util::sync::CancellationToken;

use super::{contracts::ImageOperation, source::SourceImage};
use crate::ai::{
    models::ExecutionTarget,
    providers::dashscope::{execute_image_operation, SourceImageOperation},
    transport::SecurityContext,
};

pub async fn execute(
    target: &ExecutionTarget,
    operation: &ImageOperation,
    source: &SourceImage,
    cancellation: &CancellationToken,
) -> Result<Vec<u8>, String> {
    if target.provider() != "qwen" {
        return Err(format!(
            "Configured image provider '{}' does not support source-image operations",
            target.provider()
        ));
    }
    let ctx = SecurityContext::from_target(target)
        .await
        .map_err(|error| error.to_string())?;
    let bytes = match operation {
        ImageOperation::Upscale { target_long_edge } => {
            upscale(&ctx, source, *target_long_edge, cancellation).await?
        }
        ImageOperation::MultiAngle { .. } => {
            let prompt = operation
                .view_prompt()
                .ok_or("Image view prompt is unavailable")?;
            execute_image_operation(
                &ctx,
                SourceImageOperation::View {
                    data_uri: &source.data_uri,
                    prompt: &prompt,
                },
                cancellation,
            )
            .await?
        }
    };
    Ok(bytes)
}

async fn upscale(
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
    let mut data_uri = source.data_uri.clone();
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
            move || super::source::encode_any(&candidate)
        })
        .await?;
        (width, height) = (encoded.width, encoded.height);
        let next = width.max(height);
        if next <= current {
            return Err("Upscale provider did not increase the image resolution".into());
        }
        current = next;
        data_uri = encoded.data_uri;
    }
    Ok(bytes)
}
