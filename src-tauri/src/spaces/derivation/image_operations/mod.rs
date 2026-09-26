pub(crate) mod contracts;
mod output;
mod output_file;
mod persistence;
mod provider;
mod source;
mod validation;

use tauri::{ipc::Channel, AppHandle};

pub use contracts::{CreateImageOperationRequest, ImageOperationCapabilities};

use crate::{
    ai::{
        contracts::{ExecutionCapability, RoutingPurpose},
        settings,
    },
    spaces::derivation::CompletedDerivationOutput,
};

pub async fn capabilities(app: &AppHandle) -> ImageOperationCapabilities {
    use contracts::{ImageOperationBlockReason as Reason, ImageOperationCapability as Capability};
    let configured = match resolve_target(app) {
        Err(_) => Capability::Blocked {
            reason: Reason::ConfigurationIncomplete,
        },
        Ok(target) if target.provider() != "qwen" => Capability::Blocked {
            reason: Reason::ProviderUnsupported,
        },
        Ok(target) => match crate::ai::transport::SecurityContext::from_target(&target).await {
            Ok(_) => Capability::Ready,
            Err(error) if error.code == crate::ai::error::AiErrorCode::NotConfigured => {
                Capability::Blocked {
                    reason: Reason::ConfigurationIncomplete,
                }
            }
            Err(_) => Capability::Blocked {
                reason: Reason::CredentialUnavailable,
            },
        },
    };
    ImageOperationCapabilities {
        upscale: configured,
        multi_angle: configured,
    }
}

pub async fn create(
    app: AppHandle,
    request: CreateImageOperationRequest,
    stages: Channel<String>,
) -> Result<CompletedDerivationOutput, String> {
    let target = match resolve_target(&app) {
        Ok(value) => value,
        Err(error) => return fail_pending(&app, &request, error).await,
    };
    let (cancellation, _lease) = match crate::ai::executions::acquire(request.id.clone()) {
        Ok(value) => value,
        Err(error) => return fail_pending(&app, &request, error.to_string()).await,
    };
    if let Err(error) = persistence::start(&app, &request).await {
        return fail_pending(&app, &request, error).await;
    }
    if let Err(error) = super::stage(&stages, "renderingSource") {
        return fail_running(&app, &request, error).await;
    }
    let source_path =
        match super::source_snapshot::image_path(&app, &request.id, &request.source_edit).await {
            Ok(value) => value,
            Err(error) => return fail_running(&app, &request, error).await,
        };
    let rendered = crate::blocking::run({
        let render_app = app.clone();
        let source_edit = request.source_edit.clone();
        move || crate::images::render_source_png(&render_app, &source_path, &source_edit)
    })
    .await;
    let source = match rendered {
        Ok(value) => match crate::blocking::run(move || source::encode_canonical(value)).await {
            Ok(source) => source,
            Err(error) => return fail_running(&app, &request, error).await,
        },
        Err(error) => return fail_running(&app, &request, error).await,
    };
    if let Err(error) = super::stage(&stages, "processing") {
        return fail_running(&app, &request, error).await;
    }
    let generated = provider::execute(&target, &request.operation, &source, &cancellation).await;
    let result = match generated {
        Ok(generated) => {
            let target = request.operation.target_long_edge();
            match crate::blocking::run(move || output::canonicalize(&generated, target)).await {
                Ok(image) => persistence::complete(&app, &request, image).await,
                Err(error) => Err(error),
            }
        }
        Err(error) => Err(error),
    };
    match result {
        Ok(output) => Ok(output),
        Err(error) => {
            super::failure::fail_message(&app, &request.id, &error).await?;
            Err(error)
        }
    }
}

async fn fail_running<T>(
    app: &AppHandle,
    request: &CreateImageOperationRequest,
    error: String,
) -> Result<T, String> {
    super::failure::fail_message(app, &request.id, &error).await?;
    Err(error)
}

async fn fail_pending<T>(
    app: &AppHandle,
    request: &CreateImageOperationRequest,
    error: String,
) -> Result<T, String> {
    super::failure::fail_pending(
        app,
        &request.space_id,
        &request.output_node_id,
        super::failure::PendingNodeState::Processing,
    )
    .await?;
    Err(error)
}

fn resolve_target(app: &AppHandle) -> Result<crate::ai::models::ExecutionTarget, String> {
    let selection = settings::default_selection(app, RoutingPurpose::ImageGeneration)
        .map_err(|error| error.to_string())?;
    settings::resolve(app, selection, ExecutionCapability::ImageGeneration)
        .map_err(|error| error.to_string())
}
