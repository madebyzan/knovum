use super::{
    agent, contracts::DeriveImageRequest, local, output, persistence, provider, routing, source,
};
use crate::{foundation::app_error::AppError, spaces::derivation::CompletedDerivationOutput};
use tauri::{ipc::Channel, AppHandle};

pub(crate) async fn derive_for_action(
    app: AppHandle,
    request: DeriveImageRequest,
    stages: Channel<String>,
    authority: &super::super::action_execution::ExecutionAuthority,
    cancellation: &tokio_util::sync::CancellationToken,
) -> Result<CompletedDerivationOutput, AppError> {
    let target = match routing::resolve(&app, &request.operation, &request.execution) {
        Ok(value) => value,
        Err(error) => return fail_pending(&app, &request, error).await,
    };
    if let Err(error) = persistence::start(
        &app,
        &request,
        &target.engine(&request.operation)?,
        authority,
    )
    .await
    {
        return fail_pending(&app, &request, error).await;
    }
    if let Err(error) = super::super::stage(&stages, "renderingSource") {
        return fail_running(&app, &request, error.into()).await;
    }
    let source_path =
        match super::super::source_snapshot::image_path(&app, &request.id, &request.source_edit)
            .await
        {
            Ok(value) => value,
            Err(error) => return fail_running(&app, &request, error.into()).await,
        };
    let rendered = super::super::source_snapshot::render_pinned_png(
        &app,
        &source_path,
        &request.source_revision,
        &request.source_edit,
    )
    .await;
    let source = match rendered {
        Ok(value) => source::SourceImage::from(value),
        Err(error) => return fail_running(&app, &request, error.into()).await,
    };
    if let Err(error) = super::super::stage(&stages, "processing") {
        return fail_running(&app, &request, error.into()).await;
    }
    let edited_image = async {
        let image = match &target {
            routing::ResolvedImageTarget::Local(model) => {
                super::super::stage(&stages, "preparingResources")?;
                local::execute(
                    &app,
                    *model,
                    &request.operation,
                    &source,
                    &request.id,
                    cancellation,
                )
                .await?
            }
            routing::ResolvedImageTarget::Remote(target) => {
                let bytes = match target {
                    crate::ai::tasks::TaskExecutionTarget::Model(model) => {
                        provider::execute(model, &request.operation, &source, cancellation).await?
                    }
                    crate::ai::tasks::TaskExecutionTarget::Agent { runtime_kind } => {
                        agent::execute(
                            &app,
                            *runtime_kind,
                            &request.operation,
                            &source,
                            cancellation,
                        )
                        .await?
                    }
                };
                crate::foundation::blocking::run(move || output::canonicalize(&bytes)).await?
            }
        };
        super::super::stage(&stages, "publishing")?;
        persistence::complete(&app, &request, image, authority, cancellation).await
    }
    .await;
    match edited_image {
        Ok(output) => Ok(output),
        Err(error) => {
            super::super::failure::fail_message(&app, &request.id, &error.message).await?;
            Err(error)
        }
    }
}

async fn fail_running<T>(
    app: &AppHandle,
    request: &DeriveImageRequest,
    error: AppError,
) -> Result<T, AppError> {
    super::super::failure::fail_message(app, &request.id, &error.message).await?;
    Err(error)
}

async fn fail_pending<T>(
    app: &AppHandle,
    request: &DeriveImageRequest,
    error: AppError,
) -> Result<T, AppError> {
    super::super::failure::fail_pending(
        app,
        &request.space_id,
        &request.output_node_id,
        super::super::failure::PendingNodeState::Processing,
    )
    .await?;
    Err(error)
}
