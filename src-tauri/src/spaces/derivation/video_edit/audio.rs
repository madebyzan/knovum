//! Source audio for the editor's preview. Playback streams a source in short decoded pieces around
//! the playhead, and the timeline reads a waveform that arrives as it is decoded, so neither waits
//! for — or holds in memory — a decode of the whole source. FFmpeg owns the decoding: it reads every
//! container and codec the editor admits, which the WebView's own decoders do not.

use std::{
    fs,
    io::Read,
    path::Path,
    process::{Command, Stdio},
    thread,
};

use tauri::{
    ipc::{Channel, Response},
    AppHandle,
};

use super::{
    media_source,
    waveform_peaks::{PeakReducer, PEAKS_PER_SECOND, SAMPLE_RATE},
    ReadVideoEditAudioRequest, StreamVideoEditWaveformRequest, VideoEditWaveformChunk,
};
use crate::{
    foundation::{blocking, subprocess},
    local_resources::tools,
};

const AUDIO_KINDS: &[&str] = &["video", "audio"];
/// Longest stretch one read decodes; the preview asks for pieces this long or shorter.
const MAX_READ_MS: u32 = 10_000;
const SAMPLE_RATES: std::ops::RangeInclusive<u32> = 8_000..=192_000;
/// Peaks per waveform message, so a lane fills in as the decode moves through the source.
const MESSAGE_PEAKS: usize = 3_000;
const WAVEFORM_RECIPE: &str = "s16le-8000-mono-100";
const DIAGNOSTIC_BYTES: u64 = 64 * 1024;

/// Decodes `[from, from + length)` of a source's first audio stream to interleaved stereo `f32le`
/// at the preview's sample rate. FFmpeg's input seek decodes and discards up to the exact position,
/// so consecutive pieces join without a gap or an overlap.
pub async fn read(app: AppHandle, request: ReadVideoEditAudioRequest) -> Result<Response, String> {
    if request.length_ms == 0 || request.length_ms > MAX_READ_MS {
        return Err("Audio read length is invalid".into());
    }
    if !SAMPLE_RATES.contains(&request.sample_rate) {
        return Err("Audio sample rate is invalid".into());
    }
    let source = media_source::resolve(&app, &request.source, AUDIO_KINDS).await?;
    let ffmpeg = tools::resolve_ffmpeg(&app)?.ok_or("FFmpeg is required to play source audio")?;
    let pcm = blocking::run(move || decode_pcm(&ffmpeg, &source, &request)).await?;
    Ok(Response::new(pcm))
}

/// Streams the source's peaks to `peaks` as they are decoded, then keeps them for the revision so
/// opening the source again delivers them at once.
pub async fn stream_waveform(
    app: AppHandle,
    request: StreamVideoEditWaveformRequest,
    peaks: Channel<VideoEditWaveformChunk>,
) -> Result<(), String> {
    let source = media_source::resolve(&app, &request.source, AUDIO_KINDS).await?;
    let cache = media_source::cache_path(
        &app,
        "video-edit-waveforms",
        &source,
        &request.source.source_revision,
        WAVEFORM_RECIPE,
    )?;
    let ffmpeg = tools::resolve_ffmpeg(&app)?.ok_or("FFmpeg is required to draw waveforms")?;
    blocking::run(move || {
        if let Some(cached) = fs::read(&cache).ok().filter(|cached| !cached.is_empty()) {
            return deliver(&peaks, 0, &cached);
        }
        let decoded = decode_peaks(&ffmpeg, &source, &peaks)?;
        media_source::store(&cache, &decoded)
    })
    .await
}

fn ffmpeg_command(ffmpeg: &Path) -> Command {
    let mut command = Command::new(ffmpeg);
    command
        .args(["-nostdin", "-hide_banner", "-loglevel", "error"])
        .stdin(Stdio::null());
    // Preview reads run every few seconds while playing; none of them may flash a console window.
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
    }
    command
}

fn decode_pcm(
    ffmpeg: &Path,
    source: &str,
    request: &ReadVideoEditAudioRequest,
) -> Result<Vec<u8>, String> {
    let output = ffmpeg_command(ffmpeg)
        .arg("-ss")
        .arg(seconds(request.from_ms))
        .args(["-i", source, "-t"])
        .arg(seconds(request.length_ms))
        .args(["-map", "0:a:0", "-vn", "-ac", "2", "-ar"])
        .arg(request.sample_rate.to_string())
        .args(["-f", "f32le", "pipe:1"])
        .output()
        .map_err(|error| format!("Unable to read source audio: {error}"))?;
    if !output.status.success() {
        return Err(failure(&String::from_utf8_lossy(&output.stderr)));
    }
    Ok(output.stdout)
}

fn decode_peaks(
    ffmpeg: &Path,
    source: &str,
    peaks: &Channel<VideoEditWaveformChunk>,
) -> Result<Vec<u8>, String> {
    let mut child = ffmpeg_command(ffmpeg)
        .args(["-i", source, "-map", "0:a:0", "-vn", "-ac", "1", "-ar"])
        .arg(SAMPLE_RATE.to_string())
        .args(["-f", "s16le", "pipe:1"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("Unable to read source audio: {error}"))?;
    let stderr = child.stderr.take().ok_or("FFmpeg stderr is unavailable")?;
    let diagnostics = thread::spawn(move || {
        let mut text = String::new();
        let _ = stderr.take(DIAGNOSTIC_BYTES).read_to_string(&mut text);
        text
    });
    let mut stdout = child.stdout.take().ok_or("FFmpeg stdout is unavailable")?;
    let streamed = stream_peaks(&mut stdout, peaks);
    if streamed.is_err() {
        let _ = child.kill();
    }
    let status = child.wait().map_err(|error| error.to_string())?;
    let diagnostics = diagnostics.join().unwrap_or_default();
    let decoded = streamed?;
    if !status.success() {
        return Err(failure(&diagnostics));
    }
    Ok(decoded)
}

fn stream_peaks(
    stdout: &mut impl Read,
    peaks: &Channel<VideoEditWaveformChunk>,
) -> Result<Vec<u8>, String> {
    let mut reducer = PeakReducer::default();
    let mut delivered = Vec::new();
    let mut pending = Vec::new();
    let mut buffer = vec![0; 64 * 1024];
    loop {
        let read = stdout
            .read(&mut buffer)
            .map_err(|error| format!("Unable to read decoded audio: {error}"))?;
        if read == 0 {
            break;
        }
        reducer.push(&buffer[..read]);
        pending.append(&mut reducer.take());
        if pending.len() >= MESSAGE_PEAKS {
            deliver(peaks, delivered.len(), &pending)?;
            delivered.append(&mut pending);
        }
    }
    pending.append(&mut reducer.finish());
    if !pending.is_empty() {
        deliver(peaks, delivered.len(), &pending)?;
        delivered.append(&mut pending);
    }
    Ok(delivered)
}

fn deliver(
    channel: &Channel<VideoEditWaveformChunk>,
    offset: usize,
    peaks: &[u8],
) -> Result<(), String> {
    channel
        .send(VideoEditWaveformChunk {
            offset: u32::try_from(offset).map_err(|error| error.to_string())?,
            rate: PEAKS_PER_SECOND,
            peaks: peaks.to_vec(),
        })
        .map_err(|error| error.to_string())
}

fn failure(stderr: &str) -> String {
    let lines: Vec<String> = stderr.lines().map(str::to_owned).collect();
    subprocess::diagnostic_error(&lines, "Source audio could not be read")
}

fn seconds(ms: u32) -> String {
    format!("{}.{:03}", ms / 1_000, ms % 1_000)
}
