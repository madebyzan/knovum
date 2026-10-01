use crate::app_error::AppError;
mod agent;
pub(crate) mod contracts;
mod mask;
pub(crate) use agent::discard_interrupted;
pub(super) mod output;
mod output_file;
mod persistence;
mod provider;
mod routing;
mod source;
mod upscale;
mod validation;

use tauri::{ipc::Channel, AppHandle};

pub use contracts::{DeriveImageRequest, ImageOperationCapabilities};

use crate::{
    ai::{
        contracts::{ExecutionCapability, RoutingPurpose},
        settings,
    },
    spaces::derivation::CompletedDerivationOutput,
};

pub async fn capabilities(app: &AppHandle) -> ImageOperationCapabilities {
    use contracts::{ImageOperationBlockReason as Reason, ImageOperationCapability as Capability};
    let target = match resolve_target(app) {
        Ok(target) => target,
        Err(_) => {
            return ImageOperationCapabilities::all(Capability::Blocked {
                reason: Reason::ConfigurationIncomplete,
            })
        }
    };
    let crate::ai::tasks::TaskExecutionTarget::Model(target) = target else {
        return ImageOperationCapabilities::all(Capability::Unverified);
    };
    let operations = routing::supported_operations(routing::backend(&target));
    let configured = match crate::ai::transport::SecurityContext::from_target(&target).await {
        Ok(_) => Capability::Ready,
        Err(error) if error.code == "not_configured" => Capability::Blocked {
            reason: Reason::ConfigurationIncomplete,
        },
        Err(_) => Capability::Blocked {
            reason: Reason::CredentialUnavailable,
        },
    };
    let supported = |enabled| {
        if enabled {
            configured
        } else {
            Capability::Blocked {
                reason: Reason::ProviderUnsupported,
            }
        }
    };
    ImageOperationCapabilities {
        upscale: supported(operations.upscale),
        multi_angle: supported(operations.multi_angle),
        masked_removal: supported(operations.masked_removal),
        prompt_removal: supported(operations.prompt_removal),
    }
}

impl ImageOperationCapabilities {
    fn all(capability: contracts::ImageOperationCapability) -> Self {
        Self {
            upscale: capability,
            multi_angle: capability,
            masked_removal: capability,
            prompt_removal: capability,
        }
    }
}

pub async fn derive(
    app: AppHandle,
    request: DeriveImageRequest,
    stages: Channel<String>,
) -> Result<CompletedDerivationOutput, AppError> {
    let target = match resolve_target(&app) {
        Ok(value) => value,
        Err(error) => return fail_pending(&app, &request, error).await,
    };
    if let crate::ai::tasks::TaskExecutionTarget::Model(model) = &target {
        if !routing::supported_operations(routing::backend(model)).allows(&request.operation) {
            return fail_pending(
                &app,
                &request,
                "Configured image model does not support this image operation".into(),
            )
            .await;
        }
    }
    let (cancellation, _lease) = match crate::ai::executions::acquire(request.id.clone()) {
        Ok(value) => value,
        Err(error) => return fail_pending(&app, &request, error.into()).await,
    };
    if let Err(error) = persistence::start(&app, &request, &target).await {
        return fail_pending(&app, &request, error).await;
    }
    if let Err(error) = super::stage(&stages, "renderingSource") {
        return fail_running(&app, &request, error.into()).await;
    }
    let source_path =
        match super::source_snapshot::image_path(&app, &request.id, &request.source_edit).await {
            Ok(value) => value,
            Err(error) => return fail_running(&app, &request, error.into()).await,
        };
    let rendered = crate::blocking::run({
        let render_app = app.clone();
        let source_edit = request.source_edit.clone();
        move || {
            let source = crate::spaces::assets::authorize_spaces_asset(
                &render_app,
                std::path::Path::new(&source_path),
            )?;
            crate::images::render_source_png(&source, &source_edit)
        }
    })
    .await;
    let source = match rendered {
        Ok(value) => match crate::blocking::run(move || source::from_rendered(value)).await {
            Ok(source) => source,
            Err(error) => return fail_running(&app, &request, error.into()).await,
        },
        Err(error) => return fail_running(&app, &request, error.into()).await,
    };
    if let Err(error) = super::stage(&stages, "processing") {
        return fail_running(&app, &request, error.into()).await;
    }
    let generated = match &target {
        crate::ai::tasks::TaskExecutionTarget::Model(model) => {
            provider::execute(
                model,
                routing::backend(model),
                &request.operation,
                &source,
                &cancellation,
            )
            .await
        }
        crate::ai::tasks::TaskExecutionTarget::Agent { runtime_kind } => {
            agent::execute(
                &app,
                *runtime_kind,
                &request.operation,
                &source,
                &cancellation,
            )
            .await
        }
    };
    let edited_image = match generated {
        Ok(generated) => {
            let target = request.operation.target_long_edge();
            match crate::blocking::run(move || output::canonicalize(&generated, target)).await {
                Ok(image) => persistence::complete(&app, &request, image, &cancellation).await,
                Err(error) => Err(error.into()),
            }
        }
        Err(error) => Err(error),
    };
    match edited_image {
        Ok(output) => Ok(output),
        Err(error) => {
            super::failure::fail_message(&app, &request.id, &error.message).await?;
            Err(error)
        }
    }
}

async fn fail_running<T>(
    app: &AppHandle,
    request: &DeriveImageRequest,
    error: AppError,
) -> Result<T, AppError> {
    super::failure::fail_message(app, &request.id, &error.message).await?;
    Err(error)
}

async fn fail_pending<T>(
    app: &AppHandle,
    request: &DeriveImageRequest,
    error: AppError,
) -> Result<T, AppError> {
    super::failure::fail_pending(
        app,
        &request.space_id,
        &request.output_node_id,
        super::failure::PendingNodeState::Processing,
    )
    .await?;
    Err(error)
}

fn resolve_target(app: &AppHandle) -> Result<crate::ai::tasks::TaskExecutionTarget, AppError> {
    let selection = settings::default_selection(app, RoutingPurpose::ImageGeneration)?;
    settings::resolve_task(app, selection, ExecutionCapability::ImageGeneration)
}
