use std::path::{Path, PathBuf};

use tauri::{AppHandle, Runtime};
use tokio_util::sync::CancellationToken;

use super::super::encoding::{encode, normalize_sources, write_concat_file, EncodeRequest};
use super::ChapterPublicationRequest;
use crate::{
    local_resources::tools,
    spaces::{assets, speech::publication::hash_file},
};

pub(super) struct RenderedChapter {
    pub output: PathBuf,
    pub part: PathBuf,
    pub sha256: String,
    pub sample_count: u64,
}

pub(super) async fn render<R: Runtime>(
    app: &AppHandle<R>,
    request: &ChapterPublicationRequest<'_>,
    output: PathBuf,
    part: PathBuf,
    work: &Path,
    cancellation: &CancellationToken,
) -> Result<RenderedChapter, String> {
    let sources = request
        .renders
        .iter()
        .map(|render| {
            let path = assets::authorize_spaces_asset(app, Path::new(&render.path))?;
            if hash_file(&path)? != render.sha256 {
                return Err("Speech segment digest differs from its render metadata".into());
            }
            Ok(path)
        })
        .collect::<Result<Vec<_>, String>>()?;
    let ffmpeg = tools::resolve_tool(app, "KNOVUM_FFMPEG_PATH", tools::ffmpeg_name())?
        .ok_or("The bundled FFmpeg tool is unavailable")?;
    let normalized = crate::blocking::run({
        let ffmpeg = ffmpeg.clone();
        let sources = sources.clone();
        let renders = request.renders.to_vec();
        let work = work.to_owned();
        let cancellation = cancellation.clone();
        let sample_rate = request.sample_rate;
        move || {
            normalize_sources(
                &ffmpeg,
                &sources,
                &renders,
                sample_rate,
                &work,
                &cancellation,
            )
        }
    })
    .await?;
    let sample_count = normalized.iter().try_fold(0_u64, |total, source| {
        total
            .checked_add(source.sample_count)
            .ok_or_else(|| "Chapter sample count overflow".to_owned())
    })?;
    let concat = write_concat_file(work, &normalized)?;
    crate::blocking::run({
        let part = part.clone();
        let cancellation = cancellation.clone();
        let sample_rate = request.sample_rate;
        move || {
            encode(
                EncodeRequest {
                    ffmpeg: &ffmpeg,
                    concat_path: &concat,
                    output: &part,
                    format: "wav",
                    sample_rate,
                    audiobook: None,
                    measurement: None,
                    metadata: None,
                },
                &cancellation,
            )
        }
    })
    .await?;
    for (source, render) in sources.iter().zip(request.renders) {
        if cancellation.is_cancelled() {
            return Err("cancelled".into());
        }
        if hash_file(source)? != render.sha256 {
            return Err(
                "Speech segment changed while its audiobook chapter was being assembled".into(),
            );
        }
    }
    Ok(RenderedChapter {
        sha256: hash_file(&part)?,
        output,
        part,
        sample_count,
    })
}
