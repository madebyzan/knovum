use super::super::{
    cancellation,
    publication::hash_file,
    video_publication::{media_pipeline, policy},
};
use super::{contracts::ExportVideoAudioTracksRequest, files, repository};
use crate::{
    app_error::AppError,
    file_inspection::verify_content_revision,
    local_resources::tools,
    spaces::{assets, db},
};
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
    process::Command,
};
use tauri::AppHandle;

struct Track {
    path: PathBuf,
    sha256: String,
    label: String,
    language: String,
}

#[tauri::command]
pub async fn export_video_audio_tracks(
    app: AppHandle,
    request: ExportVideoAudioTracksRequest,
) -> Result<(), AppError> {
    let lease = cancellation::acquire(&request.job_id)?;
    if request.revision_ids.is_empty()
        || request.revision_ids.len() > 8
        || request.revision_ids.iter().collect::<HashSet<_>>().len() != request.revision_ids.len()
    {
        return Err("Select between one and eight distinct published dubbing revisions".into());
    }
    let destination = Path::new(&request.destination_path).to_owned();
    files::authorize(&app, &destination, "mp4")?;
    let pool = db::pool(&app).await?;
    let catalog = repository::tracks(pool, &request.project_id).await?;
    let mut video = None;
    let mut tracks = Vec::with_capacity(request.revision_ids.len());
    for revision_id in &request.revision_ids {
        let revision = repository::load(pool, &request.project_id, revision_id).await?;
        let config = revision.config()?;
        if video.as_ref().is_some_and(|source| source != &config.video) {
            return Err("Audio tracks must share the same immutable source video".into());
        }
        video = Some(config.video);
        let (path, sha256) = revision.published_output()?;
        let metadata = catalog
            .iter()
            .find(|track| &track.revision_id == revision_id)
            .ok_or("Published track metadata is unavailable")?;
        tracks.push(Track {
            path: assets::authorize_spaces_asset(&app, Path::new(path))?,
            sha256: sha256.to_owned(),
            label: format!("{} · v{}", metadata.language, metadata.revision),
            language: metadata.language.clone(),
        });
    }
    let video = video.ok_or("Source video is unavailable")?;
    let source = policy::verified_source(&app, &video.asset_path, &video.source_revision)?;
    let work = files::workspace(
        &app,
        &request.project_id,
        &request.revision_ids[0],
        &request.job_id,
    )?;
    let cancellation = lease.token().clone();
    Ok(crate::blocking::run(move || -> Result<_, String> {
        let result = (|| {
            for track in &tracks {
                verify_track(track)?;
            }
            let ffmpeg = tools::resolve_tool(&app, "KNOVUM_FFMPEG_PATH", tools::ffmpeg_name())?
                .ok_or("Bundled FFmpeg is unavailable")?;
            let output = work.join("tracks.mp4");
            mux_tracks(
                TrackMuxRequest {
                    ffmpeg: &ffmpeg,
                    source: &source,
                    tracks: &tracks,
                    output: &output,
                    duration_ms: video.duration_ms,
                    retain_original_track: request.retain_original_track,
                },
                &cancellation,
            )?;
            verify_content_revision(&source, &video.source_revision)?;
            for track in &tracks {
                verify_track(track)?;
            }
            files::commit(&output, &destination, &cancellation)
        })();
        files::finish(result, &work)
    })
    .await?)
}
fn verify_track(track: &Track) -> Result<(), String> {
    if hash_file(&track.path)? == track.sha256 {
        Ok(())
    } else {
        Err("Published dubbing track changed during export".into())
    }
}
fn language_tag(language: &str) -> &str {
    match language.split('-').next().unwrap_or(language) {
        "zh" => "zho",
        "en" => "eng",
        "ja" => "jpn",
        "ko" => "kor",
        "es" => "spa",
        "fr" => "fra",
        "de" => "deu",
        "it" => "ita",
        "pt" => "por",
        "ru" => "rus",
        "ar" => "ara",
        "hi" => "hin",
        _ => "und",
    }
}

struct TrackMuxRequest<'a> {
    ffmpeg: &'a Path,
    source: &'a Path,
    tracks: &'a [Track],
    output: &'a Path,
    duration_ms: u64,
    retain_original_track: bool,
}
fn mux_tracks(
    request: TrackMuxRequest<'_>,
    cancellation: &tokio_util::sync::CancellationToken,
) -> Result<(), String> {
    let TrackMuxRequest {
        ffmpeg,
        source,
        tracks,
        output,
        duration_ms,
        retain_original_track,
    } = request;
    let mut command = Command::new(ffmpeg);
    command
        .args(["-hide_banner", "-loglevel", "error", "-nostdin", "-n", "-i"])
        .arg(source);
    for track in tracks {
        command.arg("-i").arg(&track.path);
    }
    if retain_original_track {
        command.args(["-filter_complex", "[0:a:0]apad[original]"]);
    }
    command.args(["-map", "0:v:0"]);
    for (index, track) in tracks.iter().enumerate() {
        command.args([
            "-map",
            &format!("{}:a:0", index + 1),
            &format!("-metadata:s:a:{index}"),
            &format!("title={}", track.label),
            &format!("-metadata:s:a:{index}"),
            &format!("language={}", language_tag(&track.language)),
            &format!("-disposition:a:{index}"),
            if index == 0 { "default" } else { "0" },
        ]);
    }
    if retain_original_track {
        command.args([
            "-map",
            "[original]",
            &format!("-metadata:s:a:{}", tracks.len()),
            "title=Original",
            &format!("-disposition:a:{}", tracks.len()),
            "0",
        ]);
    }
    command
        .args([
            "-c:v",
            "copy",
            "-c:a",
            "aac",
            "-ac",
            "2",
            "-b:a",
            "192k",
            "-movflags",
            "+faststart",
            "-f",
            "mp4",
        ])
        .args(["-t", &format!("{:.3}", duration_ms as f64 / 1000.0)])
        .arg(output);
    media_pipeline::run_ffmpeg(
        &mut command,
        cancellation,
        "Could not export video audio tracks",
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[ignore = "Requires the explicit FFmpeg test binary"]
    fn multiple_language_tracks_preserve_duration_and_stream_metadata() {
        let work =
            std::env::temp_dir().join(format!("knovum-tracks-test-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&work).unwrap();
        let tool =
            PathBuf::from(std::env::var_os("KNOVUM_FFMPEG_PATH").expect("Set KNOVUM_FFMPEG_PATH"));
        let source = work.join("source.mp4");
        assert!(Command::new(&tool)
            .args([
                "-hide_banner",
                "-loglevel",
                "error",
                "-f",
                "lavfi",
                "-i",
                "color=size=32x32:rate=10:duration=2",
                "-f",
                "lavfi",
                "-i",
                "sine=duration=1",
                "-c:v",
                "mpeg4",
                "-c:a",
                "aac"
            ])
            .arg(&source)
            .status()
            .unwrap()
            .success());
        let audio = work.join("track.wav");
        assert!(Command::new(&tool)
            .args([
                "-hide_banner",
                "-loglevel",
                "error",
                "-f",
                "lavfi",
                "-i",
                "sine=duration=2",
                "-c:a",
                "pcm_f32le"
            ])
            .arg(&audio)
            .status()
            .unwrap()
            .success());
        let tracks = [
            Track {
                path: audio.clone(),
                sha256: hash_file(&audio).unwrap(),
                label: "Chinese".into(),
                language: "zh-CN".into(),
            },
            Track {
                path: audio,
                sha256: String::new(),
                label: "English".into(),
                language: "en".into(),
            },
        ];
        let output = work.join("delivery.mp4");
        let cancel = tokio_util::sync::CancellationToken::new();
        mux_tracks(
            TrackMuxRequest {
                ffmpeg: &tool,
                source: &source,
                tracks: &tracks,
                output: &output,
                duration_ms: 2000,
                retain_original_track: true,
            },
            &cancel,
        )
        .unwrap();
        for stream in ["0:v:0", "0:a:0", "0:a:1", "0:a:2"] {
            let duration =
                media_pipeline::verified_duration_us(&tool, &output, stream, 2000, &cancel)
                    .unwrap();
            assert!(
                (1_900_000..=2_100_000).contains(&duration),
                "Wrong {stream} duration: {duration}"
            );
        }
        let inspection = Command::new(&tool)
            .args(["-hide_banner", "-nostdin", "-i"])
            .arg(&output)
            .args(["-map", "0:a", "-f", "null", "-"])
            .output()
            .unwrap();
        assert!(inspection.status.success());
        let diagnostic = String::from_utf8(inspection.stderr).unwrap();
        assert!(diagnostic.contains("(zho)"));
        assert!(diagnostic.contains("(eng)"));
        assert!(diagnostic.contains("Original"));
        std::fs::remove_dir_all(work).unwrap();
    }
}
