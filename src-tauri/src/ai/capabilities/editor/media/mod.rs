mod directives;

use std::{collections::BTreeMap, path::Path, sync::Arc};

use tauri::{ipc::Channel, AppHandle};
use tokio::{sync::Semaphore, task::JoinSet};
use tokio_util::sync::CancellationToken;

use self::directives::{
    collect_directives, parse_directive, EditorMediaKind, MediaGenerationDirective,
};
use crate::{
    ai::{
        capabilities::{
            image::{self, ImageGenerationRequest},
            video::{self, VideoGenerationRequest},
        },
        channel::send_channel_event,
        contracts::{ExecutionCapability, RoutingPurpose},
        settings,
        tasks::{stream::TaskStreamEvent, TaskExecutionTarget},
    },
    foundation::app_error::{AppError, AppResult},
    spaces::assets::{
        image::persist_detected_bytes,
        markdown_media::{self, persist_detected_video_bytes},
        to_webview_url,
    },
};

const MAX_CONCURRENT_MEDIA_JOBS: usize = 3;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct EditorMediaCapabilities {
    pub(crate) image: bool,
    pub(crate) video: bool,
}

impl EditorMediaCapabilities {
    pub(crate) fn detect(app: &AppHandle) -> Self {
        Self {
            image: routing_available(
                app,
                RoutingPurpose::ImageGeneration,
                ExecutionCapability::ImageGeneration,
            ),
            video: routing_available(
                app,
                RoutingPurpose::VideoGeneration,
                ExecutionCapability::VideoGeneration,
            ),
        }
    }

    pub(crate) const fn any(self) -> bool {
        self.image || self.video
    }

    pub(crate) const fn supports(self, kind: EditorMediaKind) -> bool {
        match kind {
            EditorMediaKind::Image => self.image,
            EditorMediaKind::Video => self.video,
        }
    }

    pub(crate) fn prompt_guidance(self) -> String {
        if !self.any() {
            return "\nNever invent fake local file paths or broken placeholder media URLs."
                .to_owned();
        }
        let mut lines = vec![
            "\nYou can author complete multimodal Markdown articles by embedding generated visual media wherever it enriches the article (cover, chapter illustrations, visual examples, or motion clips):".to_owned(),
        ];
        if self.image {
            lines.push(
                "- Images: `![descriptive caption](knovum-gen://image?prompt=<URL-encoded detailed visual prompt>&aspect=16:9)` (supported aspect ratios: `1:1`, `3:2`, `2:3`, `4:3`, `3:4`, `16:9`, `9:16`).".to_owned(),
            );
        }
        if self.video {
            lines.push(
                "- Videos: `![descriptive caption](knovum-gen://video?prompt=<URL-encoded detailed motion prompt>&aspect=16:9&duration=5)` (duration: `4`–`10` seconds).".to_owned(),
            );
        }
        lines.push(
            "Knovum generates and embeds each directive automatically in parallel. Use rich, coherent visual prompts when media fits the request, and never invent fake local file paths or broken URLs.".to_owned(),
        );
        lines.join("\n")
    }
}

fn routing_available(
    app: &AppHandle,
    purpose: RoutingPurpose,
    capability: ExecutionCapability,
) -> bool {
    let Ok(selection) = settings::default_selection(app, purpose) else {
        return false;
    };
    settings::resolve_task(app, selection, capability).is_ok()
}

/// Materializes every `knovum-gen://<kind>?...` directive emitted by a built-in writing model
/// into real persisted Space assets (`asset://localhost/...`) using bounded parallel execution.
pub(crate) async fn materialize_generated_media(
    app: &AppHandle,
    markdown: &str,
    capabilities: EditorMediaCapabilities,
    events: &Channel<TaskStreamEvent>,
    cancellation: &CancellationToken,
) -> AppResult<String> {
    let directives: Vec<MediaGenerationDirective> = collect_directives(markdown)
        .into_iter()
        .filter(|directive| capabilities.supports(directive.kind))
        .collect();
    if directives.is_empty() {
        return Ok(markdown.to_owned());
    }

    let targets = resolve_targets(app, &directives)?;
    let total = directives.len();
    let semaphore = Arc::new(Semaphore::new(MAX_CONCURRENT_MEDIA_JOBS));
    let mut tasks: JoinSet<AppResult<(MediaGenerationDirective, String)>> = JoinSet::new();

    for (index, directive) in directives.into_iter().enumerate() {
        let app_handle = app.clone();
        let target = targets.target_for(directive.kind)?.clone();
        let events_channel = events.clone();
        let token = cancellation.clone();
        let permits = Arc::clone(&semaphore);
        tasks.spawn(async move {
            let _permit = permits
                .acquire_owned()
                .await
                .map_err(|_| AppError::internal("Media generation semaphore closed"))?;
            if token.is_cancelled() {
                return Err(AppError::cancelled("AI task was cancelled"));
            }
            let _ = send_channel_event(
                &events_channel,
                TaskStreamEvent::ThinkingDelta {
                    text: format!(
                        "Generating {} ({}/{total})...",
                        directive.kind.progress_label(),
                        index + 1
                    ),
                },
            );
            let url = generate_and_persist(&app_handle, &target, &directive, &token).await?;
            Ok((directive, url))
        });
    }

    let mut urls_by_directive: BTreeMap<MediaGenerationDirective, String> = BTreeMap::new();
    while let Some(joined) = tasks.join_next().await {
        match joined {
            Ok(Ok((directive, url))) => {
                urls_by_directive.insert(directive, url);
            }
            Ok(Err(error)) => {
                tasks.abort_all();
                return Err(error);
            }
            Err(join_error) => {
                tasks.abort_all();
                return Err(AppError::internal(format!(
                    "Media generation task failed: {join_error}"
                )));
            }
        }
    }

    markdown_media::rewrite_markdown_media(markdown, |dest_url| {
        Ok(parse_directive(dest_url)
            .and_then(|directive| urls_by_directive.get(&directive).cloned()))
    })
    .map_err(AppError::internal)
}

struct ResolvedMediaTargets {
    image: Option<TaskExecutionTarget>,
    video: Option<TaskExecutionTarget>,
}

impl ResolvedMediaTargets {
    fn target_for(&self, kind: EditorMediaKind) -> AppResult<&TaskExecutionTarget> {
        let target = match kind {
            EditorMediaKind::Image => self.image.as_ref(),
            EditorMediaKind::Video => self.video.as_ref(),
        };
        target.ok_or_else(|| AppError::invalid("Media generation target is not configured"))
    }
}

fn resolve_targets(
    app: &AppHandle,
    directives: &[MediaGenerationDirective],
) -> AppResult<ResolvedMediaTargets> {
    let needs_image = directives.iter().any(|d| d.kind == EditorMediaKind::Image);
    let needs_video = directives.iter().any(|d| d.kind == EditorMediaKind::Video);
    let image = if needs_image {
        let selection = settings::default_selection(app, RoutingPurpose::ImageGeneration)?;
        Some(settings::resolve_task(
            app,
            selection,
            ExecutionCapability::ImageGeneration,
        )?)
    } else {
        None
    };
    let video = if needs_video {
        let selection = settings::default_selection(app, RoutingPurpose::VideoGeneration)?;
        Some(settings::resolve_task(
            app,
            selection,
            ExecutionCapability::VideoGeneration,
        )?)
    } else {
        None
    };
    Ok(ResolvedMediaTargets { image, video })
}

async fn generate_and_persist(
    app: &AppHandle,
    target: &TaskExecutionTarget,
    directive: &MediaGenerationDirective,
    cancellation: &CancellationToken,
) -> AppResult<String> {
    match directive.kind {
        EditorMediaKind::Image => {
            let bytes = generate_image_bytes(app, target, directive, cancellation).await?;
            let app_handle = app.clone();
            let managed = crate::foundation::blocking::run(move || {
                persist_detected_bytes(&app_handle, bytes)
            })
            .await
            .map_err(AppError::internal)?;
            to_webview_url(Path::new(&managed)).map_err(AppError::internal)
        }
        EditorMediaKind::Video => {
            let bytes = generate_video_bytes(app, target, directive, cancellation).await?;
            let app_handle = app.clone();
            let managed = crate::foundation::blocking::run(move || {
                persist_detected_video_bytes(&app_handle, bytes)
            })
            .await
            .map_err(AppError::internal)?;
            to_webview_url(Path::new(&managed)).map_err(AppError::internal)
        }
    }
}

async fn generate_image_bytes(
    app: &AppHandle,
    target: &TaskExecutionTarget,
    directive: &MediaGenerationDirective,
    cancellation: &CancellationToken,
) -> AppResult<Vec<u8>> {
    match target {
        TaskExecutionTarget::Model(model_target) => {
            let generated = image::generate(
                ImageGenerationRequest {
                    target: model_target.clone(),
                    prompt: directive.prompt.clone(),
                    aspect_ratio: Some(directive.aspect.clone()),
                    references: Vec::new(),
                },
                cancellation,
            )
            .await?;
            Ok(generated.bytes)
        }
        TaskExecutionTarget::Agent { runtime_kind } => {
            crate::external_agents::task::artifact(
                app,
                *runtime_kind,
                format!(
                    "Create the requested illustration as PNG with aspect ratio {}.\n{}",
                    directive.aspect, directive.prompt
                ),
                Vec::new(),
                crate::external_agents::task::ArtifactFormat::Png,
                cancellation.clone(),
            )
            .await
        }
    }
}

async fn generate_video_bytes(
    app: &AppHandle,
    target: &TaskExecutionTarget,
    directive: &MediaGenerationDirective,
    cancellation: &CancellationToken,
) -> AppResult<Vec<u8>> {
    match target {
        TaskExecutionTarget::Model(model_target) => {
            let generated = video::generate(
                app,
                VideoGenerationRequest {
                    target: model_target.clone(),
                    prompt: directive.prompt.clone(),
                    ratio: directive.aspect.clone(),
                    duration_seconds: directive.duration_seconds,
                    reference_images: Vec::new(),
                },
                cancellation,
                |_| Ok(()),
            )
            .await?;
            Ok(generated.bytes)
        }
        TaskExecutionTarget::Agent { runtime_kind } => {
            crate::external_agents::task::artifact(
                app,
                *runtime_kind,
                format!(
                    "Create a {} second MP4 video with aspect ratio {}.\n{}",
                    directive.duration_seconds, directive.aspect, directive.prompt
                ),
                Vec::new(),
                crate::external_agents::task::ArtifactFormat::Mp4,
                cancellation.clone(),
            )
            .await
        }
    }
}

#[cfg(test)]
mod tests;
