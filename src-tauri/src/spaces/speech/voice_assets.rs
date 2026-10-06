use crate::spaces::speech::asset_publication::PublicationOwnerKind;

use std::{
    fs,
    io::Read,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use sha2::{Digest, Sha256};
use tauri::AppHandle;
use tokio_util::sync::CancellationToken;

use crate::{
    ai::contracts::speech_wire::AudioFileReference,
    local_resources::tools,
    spaces::assets,
    subprocess::{diagnostic_error, wait_with_stderr},
};

pub(super) fn prepare_reference(
    app: &AppHandle,
    source: &Path,
    output: &Path,
    expected_byte_length: u64,
    cancellation: &CancellationToken,
) -> Result<AudioFileReference, String> {
    decode_reference(
        app,
        source,
        output,
        expected_byte_length,
        cancellation,
        None,
    )
}

pub(super) fn prepare_reference_clip(
    app: &AppHandle,
    source: &Path,
    output: &Path,
    expected_byte_length: u64,
    cancellation: &CancellationToken,
    interval: (u64, u64),
) -> Result<AudioFileReference, String> {
    if interval.1 <= interval.0 || !(5000..=30000).contains(&(interval.1 - interval.0)) {
        return Err("Reference selection must contain 5–30 seconds".into());
    }
    decode_reference(
        app,
        source,
        output,
        expected_byte_length,
        cancellation,
        Some(interval),
    )
}

fn decode_reference(
    app: &AppHandle,
    source: &Path,
    output: &Path,
    expected_byte_length: u64,
    cancellation: &CancellationToken,
    selection: Option<(u64, u64)>,
) -> Result<AudioFileReference, String> {
    let interval = selection.unwrap_or((0, 31000));
    let source = authorize_managed_source(app, source, expected_byte_length)?;
    authorize_managed_output(app, output)?;
    let digest = hash_exact(&source, expected_byte_length)?;
    let ffmpeg = tools::resolve_tool(app, "KNOVUM_FFMPEG_PATH", tools::ffmpeg_name())?
        .ok_or("The bundled FFmpeg tool is unavailable")?;
    let parent = output
        .parent()
        .ok_or("Voice reference output has no parent directory")?;
    fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    if output.exists() {
        return Err("Voice reference output path is already occupied".into());
    }
    let child = Command::new(ffmpeg)
        .args(["-hide_banner", "-loglevel", "error", "-nostdin", "-n", "-i"])
        .arg(&source)
        .arg("-ss")
        .arg(format!("{:.3}", interval.0 as f64 / 1000.0))
        .arg("-t")
        .arg(format!("{:.3}", (interval.1 - interval.0) as f64 / 1000.0))
        .args([
            "-map",
            "0:a:0",
            "-vn",
            "-ac",
            "1",
            "-ar",
            "16000",
            "-c:a",
            "pcm_s16le",
            "-f",
            "wav",
        ])
        .arg(output)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("Failed to start reference audio conversion: {error}"))?;
    let publication = (|| {
        let process_output = wait_with_stderr(child, cancellation, "cancelled", |_| Ok(()))?;
        if !process_output.status.success() {
            return Err(diagnostic_error(
                &process_output.diagnostics,
                "Reference audio conversion failed",
            ));
        }
        if hash_exact(&source, expected_byte_length)? != digest {
            return Err("Reference source changed while decoding".into());
        }
        let metadata = reference_metadata(output)?;
        if let Some((start, end)) = selection {
            let samples = u64::from(
                hound::WavReader::open(output)
                    .map_err(|error| error.to_string())?
                    .duration(),
            );
            if samples.abs_diff((end - start) * 16) > 16 {
                return Err("Reference selection extends beyond its source audio".into());
            }
        }
        Ok(metadata)
    })();
    if publication.is_err() && output.exists() {
        fs::remove_file(output).map_err(|error| error.to_string())?;
    }
    publication
}

fn authorize_managed_source(
    app: &AppHandle,
    source: &Path,
    expected_byte_length: u64,
) -> Result<PathBuf, String> {
    if expected_byte_length == 0 || expected_byte_length > 128 * 1024 * 1024 {
        return Err("Voice reference source exceeds the supported byte range".into());
    }
    let source = assets::authorize_spaces_asset(app, source)?;
    let metadata = fs::symlink_metadata(&source).map_err(|error| error.to_string())?;
    if !metadata.is_file()
        || metadata.file_type().is_symlink()
        || metadata.len() != expected_byte_length
    {
        return Err("stale_revision: voice reference changed before decoding".into());
    }
    Ok(source)
}

fn authorize_managed_output(app: &AppHandle, output: &Path) -> Result<(), String> {
    if !output.is_absolute()
        || output.components().any(|component| {
            matches!(
                component,
                std::path::Component::ParentDir | std::path::Component::CurDir
            )
        })
    {
        return Err("Voice reference output path is invalid".into());
    }
    let root = assets::asset_directory(app)?
        .canonicalize()
        .map_err(|error| error.to_string())?;
    let parent = output
        .parent()
        .ok_or("Voice reference output has no parent directory")?
        .canonicalize()
        .map_err(|error| error.to_string())?;
    if parent == root || !parent.starts_with(&root) {
        return Err("Voice reference output leaves managed storage".into());
    }
    Ok(())
}

fn hash_exact(path: &Path, expected_byte_length: u64) -> Result<String, String> {
    let mut file = fs::File::open(path).map_err(|error| error.to_string())?;
    let mut digest = Sha256::new();
    let mut buffer = [0_u8; 1024 * 1024];
    let buffer_length = u64::try_from(buffer.len())
        .map_err(|_| "Voice reference hash buffer exceeds this platform")?;
    let mut remaining = expected_byte_length;
    while remaining > 0 {
        let requested = usize::try_from(remaining.min(buffer_length))
            .map_err(|_| "Voice reference length exceeds this platform")?;
        let read = file
            .read(&mut buffer[..requested])
            .map_err(|error| error.to_string())?;
        if read == 0 {
            return Err("stale_revision: voice reference shrank while hashing".into());
        }
        digest.update(&buffer[..read]);
        remaining = remaining
            .checked_sub(u64::try_from(read).map_err(|_| "Voice reference read overflow")?)
            .ok_or("Voice reference length underflow")?;
    }
    let mut extra = [0_u8; 1];
    if file.read(&mut extra).map_err(|error| error.to_string())? != 0 {
        return Err("stale_revision: voice reference grew while hashing".into());
    }
    Ok(format!("{:x}", digest.finalize()))
}

pub(super) fn reference_metadata(path: &Path) -> Result<AudioFileReference, String> {
    let reader = hound::WavReader::open(path).map_err(|error| error.to_string())?;
    let spec = reader.spec();
    if spec.channels != 1
        || spec.sample_rate != 16_000
        || spec.bits_per_sample != 16
        || spec.sample_format != hound::SampleFormat::Int
        || !(80_000..=480_000).contains(&reader.duration())
    {
        return Err(
            "Voice reference must contain 5–30 seconds of mono PCM16 audio at 16 kHz".into(),
        );
    }
    let byte_length = fs::metadata(path).map_err(|error| error.to_string())?.len();
    Ok(AudioFileReference {
        path: path.to_string_lossy().into_owned(),
        media_type: "audio/wav".into(),
        byte_length,
        sha256: hash_exact(path, byte_length)?,
        sample_rate: 16_000,
        channels: 1,
        sample_format: "pcm_s16le".into(),
    })
}

pub(super) struct PersistedReference {
    pub path: PathBuf,
    pub publication_id: String,
}

pub(super) async fn persist_reference(
    app: &AppHandle,
    reference: &AudioFileReference,
    profile_id: &str,
    space_id: &str,
) -> Result<PersistedReference, String> {
    let path = Path::new(&reference.path);
    let source = authorize_managed_source(app, path, reference.byte_length)?;
    if hash_exact(&source, reference.byte_length)? != reference.sha256 {
        return Err("Voice candidate reference has changed".into());
    }
    let directory = assets::asset_directory(app)?.join("voices");
    fs::create_dir_all(&directory).map_err(|error| error.to_string())?;
    let root = assets::asset_directory(app)?
        .canonicalize()
        .map_err(|error| error.to_string())?;
    let directory = directory
        .canonicalize()
        .map_err(|error| error.to_string())?;
    if !directory.starts_with(&root) {
        return Err("Voice assets directory leaves managed storage".into());
    }
    let target = directory.join(format!("{profile_id}.wav"));
    let part = directory.join(format!("{profile_id}.wav.part"));
    let pool = crate::spaces::db::pool(app).await?;
    let publication_id = super::asset_publication::stage_unowned(
        pool,
        space_id,
        super::asset_publication::PendingAsset {
            owner_kind: PublicationOwnerKind::VoiceReference,
            owner_id: profile_id,
            asset_path: &target,
            part_path: Some(&part),
            sha256: &reference.sha256,
        },
    )
    .await?;
    let publication = (|| {
        copy_exact(&source, &part, reference.byte_length)?;
        if hash_exact(&source, reference.byte_length)? != reference.sha256
            || hash_exact(&part, reference.byte_length)? != reference.sha256
        {
            return Err("Voice reference digest changed while copying".into());
        }
        fs::rename(&part, &target).map_err(|error| error.to_string())?;
        Ok(target.clone())
    })();
    match publication {
        Ok(path) => Ok(PersistedReference {
            path,
            publication_id,
        }),
        Err(error) => {
            super::asset_publication::abandon(app, pool, &publication_id)
                .await
                .map_err(|cleanup| format!("{error}; reference cleanup failed: {cleanup}"))?;
            Err(error)
        }
    }
}

fn copy_exact(source: &Path, target: &Path, expected_byte_length: u64) -> Result<(), String> {
    let mut input = fs::File::open(source).map_err(|error| error.to_string())?;
    let mut output = fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(target)
        .map_err(|error| error.to_string())?;
    let mut buffer = [0_u8; 1024 * 1024];
    let buffer_length = u64::try_from(buffer.len())
        .map_err(|_| "Voice reference copy buffer exceeds this platform")?;
    let mut remaining = expected_byte_length;
    while remaining > 0 {
        let requested = usize::try_from(remaining.min(buffer_length))
            .map_err(|_| "Voice reference length exceeds this platform")?;
        let read = input
            .read(&mut buffer[..requested])
            .map_err(|error| error.to_string())?;
        if read == 0 {
            return Err("Voice reference shrank while copying".into());
        }
        std::io::Write::write_all(&mut output, &buffer[..read])
            .map_err(|error| error.to_string())?;
        remaining = remaining
            .checked_sub(u64::try_from(read).map_err(|_| "Voice reference read overflow")?)
            .ok_or("Voice reference copy length underflow")?;
    }
    let mut extra = [0_u8; 1];
    if input.read(&mut extra).map_err(|error| error.to_string())? != 0 {
        return Err("Voice reference grew while copying".into());
    }
    std::io::Write::flush(&mut output).map_err(|error| error.to_string())?;
    output.sync_all().map_err(|error| error.to_string())
}
