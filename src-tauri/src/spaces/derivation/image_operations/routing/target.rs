use super::super::contracts::{ImageExecutionTarget, ImageOperation};
use crate::{
    ai::{contracts::ExecutionCapability, settings, tasks::TaskExecutionTarget},
    foundation::app_error::AppError,
    local_resources::model_catalog::{LocalModelCapability, LocalModelId},
};
use tauri::AppHandle;

pub(crate) enum ResolvedImageTarget {
    Local(LocalModelId),
    Remote(TaskExecutionTarget),
}

pub(crate) fn local_capability(operation: &ImageOperation) -> Option<LocalModelCapability> {
    use LocalModelCapability as C;
    match operation {
        ImageOperation::Upscale { .. } => Some(C::ImageUpscaling),
        ImageOperation::RemoveObject { .. } | ImageOperation::RemoveSelectedObject { .. } => {
            Some(C::ImageInpainting)
        }
        ImageOperation::ExtractSelection { .. } => Some(C::ImageSelection),
        ImageOperation::GenerateDepth => Some(C::ImageDepth),
        ImageOperation::RemoveBackground => Some(C::ImageSegmentation),
        _ => None,
    }
}

pub(crate) fn resolve(
    app: &AppHandle,
    operation: &ImageOperation,
    execution: &ImageExecutionTarget,
) -> Result<ResolvedImageTarget, AppError> {
    operation.validate()?;
    execution.validate(operation)?;
    match execution {
        ImageExecutionTarget::Local { model, .. } => Ok(ResolvedImageTarget::Local(*model)),
        ImageExecutionTarget::Remote { target } => {
            let target = settings::resolve_task(
                app,
                target.clone().with_temperature(0.2),
                ExecutionCapability::ImageGeneration,
            )?;
            if let TaskExecutionTarget::Model(model) = &target {
                if !super::supports_remote_operations(model) {
                    return Err(
                        "Configured image model does not support this image operation".into(),
                    );
                }
            }
            Ok(ResolvedImageTarget::Remote(target))
        }
    }
}

impl ResolvedImageTarget {
    pub(crate) fn engine(&self, operation: &ImageOperation) -> Result<String, String> {
        Ok(match self {
            Self::Local(model) => format!("local:{}", model.image_spec()?),
            Self::Remote(TaskExecutionTarget::Model(model)) => {
                operation.engine(model.provider(), model.model())
            }
            Self::Remote(target) => target.engine(),
        })
    }
}
