use crate::{artifact, images::Images};
use knovum_inference_contracts::{
    runtime::{
        RuntimeCapability, RuntimeInitializeParams, RuntimeLimits, RuntimeSession, PROTOCOL_VERSION,
    },
    runtime_manifest::RuntimeManifest,
};
use sha2::{Digest, Sha256};
use std::{
    collections::HashSet,
    fs,
    io::Read,
    path::{Path, PathBuf},
};

pub(crate) struct Session {
    pub(crate) identity: RuntimeSession,
    pub(crate) work: PathBuf,
    pub(crate) engine: Engine,
}

pub(crate) enum Engine {
    Images(Images),
    Embedding(Box<crate::embedding::Embedding>),
    Transcription(Box<crate::speech::transcription::Transcriber>),
    Separation(crate::speech::separation::Separator),
    Alignment(crate::speech::alignment::Aligner),
    Synthesis(crate::speech::synthesis::Synthesizer),
    Diarization(crate::speech::diarization::Diarizer),
}

impl Session {
    pub(crate) fn initialize(params: RuntimeInitializeParams) -> Result<Self, String> {
        if params.protocol_version != PROTOCOL_VERSION
            || !artifact::valid_name(&params.session_id, 128, false)
            || params.capability_spec_id.is_empty()
        {
            return Err("runtime_mismatch: invalid worker initialization identity".into());
        }
        let work = PathBuf::from(&params.work_directory);
        if !work.is_absolute() {
            return Err("invalid_input: work directory must be absolute".into());
        }
        artifact::require_directory(&work)?;
        let runtime = VerifiedRuntime::initialize(
            Path::new(&params.runtime_manifest_path),
            &params.runtime_digest,
        )?;
        let engine = Engine::load(&params, &runtime)?;
        Ok(Self {
            work,
            engine,
            identity: RuntimeSession {
                protocol_version: PROTOCOL_VERSION,
                runtime_digest: params.runtime_digest,
                session_id: params.session_id,
                capability: params.capability,
                capability_spec_id: params.capability_spec_id,
                device_profile: params.device_profile,
                capabilities: vec![params.capability.method().into()],
                limits: RuntimeLimits::CURRENT,
            },
        })
    }
}

impl Engine {
    /// A worker freezes one capability and one model at initialization and serves nothing else for
    /// the rest of its life, so its engine is built here rather than on first use: these bytes
    /// belong to the initialization budget, and no request pays for a load it did not ask for.
    fn load(params: &RuntimeInitializeParams, runtime: &VerifiedRuntime) -> Result<Self, String> {
        use crate::speech::{
            alignment::Aligner, diarization::Diarizer, separation::Separator,
            synthesis::Synthesizer, transcription::Transcriber,
        };
        match params.capability {
            RuntimeCapability::Embedding => Ok(Self::Embedding(Box::new(
                crate::embedding::Embedding::new(params)?,
            ))),
            RuntimeCapability::SpeechTranscription => Ok(Self::Transcription(Box::new(
                Transcriber::new(params, runtime)?,
            ))),
            RuntimeCapability::SpeechSeparation => {
                Ok(Self::Separation(Separator::new(params, runtime)?))
            }
            RuntimeCapability::SpeechDiarization => {
                Ok(Self::Diarization(Diarizer::new(params, runtime)?))
            }
            RuntimeCapability::SpeechAlignment => {
                Ok(Self::Alignment(Aligner::new(params, runtime)?))
            }
            RuntimeCapability::SpeechSynthesis => {
                Ok(Self::Synthesis(Synthesizer::new(params, runtime)?))
            }
            RuntimeCapability::ImageSegmentation
            | RuntimeCapability::ImageUpscaling
            | RuntimeCapability::ImageInpainting
            | RuntimeCapability::ImageSelection
            | RuntimeCapability::ImageDepth => Ok(Self::Images(Images::new(params)?)),
        }
    }
}

/// The inference runtime as this process verified it: the supplied inventory, hashed byte for byte
/// immediately before any of it is loaded.
///
/// The host cannot stand in for this check. A hash it computes can be invalidated between its
/// verification and the worker's load, so the host verifies the one thing a worker cannot verify
/// about itself — the image it executes — and the worker verifies what it is about to load.
pub(crate) struct VerifiedRuntime {
    directory: PathBuf,
    manifest: RuntimeManifest,
}

impl VerifiedRuntime {
    pub(crate) fn initialize(manifest_path: &Path, expected_digest: &str) -> Result<Self, String> {
        #[cfg(windows)]
        if windows_version::OsVersion::current() < windows_version::OsVersion::new(10, 0, 0, 19041)
        {
            return Err(
                "unsupported_device: Rust inference requires Windows 10 2004 (build 19041) or later"
                    .into(),
            );
        }
        if !manifest_path.is_absolute() {
            return Err("runtime_mismatch: inference manifest path is relative".into());
        }
        let manifest = read_manifest(manifest_path, expected_digest)?;
        let directory = manifest_path
            .parent()
            .ok_or("runtime_mismatch: inference manifest parent is missing")?
            .to_path_buf();
        artifact::require_directory(&directory)?;
        verify_inventory(&directory, &manifest)?;
        #[cfg(windows)]
        crate::speech::ffi::prepare_search_directory(&directory)?;
        ort::init_from(directory.join(&manifest.onnx_runtime))
            .map_err(|error| format!("runtime_mismatch: {error}"))?
            .with_telemetry(false)
            .commit();
        Ok(Self {
            directory,
            manifest,
        })
    }

    /// Resolves a verified inventory member. A path the inventory does not list is refused: the
    /// runtime directory is on this worker's library search path, so an unlisted file inside it
    /// must never be loaded.
    pub(crate) fn member(&self, relative: &str) -> Result<PathBuf, String> {
        if !self.manifest.files.iter().any(|file| file.path == relative) {
            return Err("runtime_mismatch: library is absent from the verified runtime".into());
        }
        Ok(self.directory.join(relative))
    }
}

fn read_manifest(path: &Path, expected_digest: &str) -> Result<RuntimeManifest, String> {
    let mut bytes = Vec::new();
    artifact::open_regular(path)?
        .take(1_048_577)
        .read_to_end(&mut bytes)
        .map_err(|error| error.to_string())?;
    if bytes.len() > 1_048_576 || hex::encode(Sha256::digest(&bytes)) != expected_digest {
        return Err("runtime_mismatch: inference manifest digest differs".into());
    }
    let manifest: RuntimeManifest =
        serde_json::from_slice(&bytes).map_err(|error| format!("runtime_mismatch: {error}"))?;
    let target = match (std::env::consts::OS, std::env::consts::ARCH) {
        ("macos", "x86_64") => "macos-x64",
        ("macos", "aarch64") => "macos-arm64",
        ("windows", "x86_64") => "windows-x64",
        _ => return Err("unsupported_device: inference runtime target is unsupported".into()),
    };
    if manifest.schema_version != 1
        || manifest.target != target
        || manifest.files.is_empty()
        || manifest.files.len() > 256
        || !manifest
            .files
            .iter()
            .any(|file| file.path == manifest.onnx_runtime)
    {
        return Err("runtime_mismatch: inference runtime manifest is invalid".into());
    }
    Ok(manifest)
}

fn verify_inventory(directory: &Path, manifest: &RuntimeManifest) -> Result<(), String> {
    let mut actual = 0;
    for entry in walkdir::WalkDir::new(directory).follow_links(false) {
        let entry = entry.map_err(|error| format!("runtime_mismatch: {error}"))?;
        let metadata = fs::symlink_metadata(entry.path()).map_err(|error| error.to_string())?;
        if metadata.file_type().is_symlink() {
            return Err("runtime_mismatch: inference runtime contains a link".into());
        }
        #[cfg(windows)]
        {
            use std::os::windows::fs::MetadataExt;
            if metadata.file_attributes()
                & windows_sys::Win32::Storage::FileSystem::FILE_ATTRIBUTE_REPARSE_POINT
                != 0
            {
                return Err("runtime_mismatch: inference runtime contains a reparse point".into());
            }
        }
        if metadata.is_dir() {
            continue;
        }
        let relative = entry
            .path()
            .strip_prefix(directory)
            .map_err(|error| error.to_string())?
            .to_string_lossy()
            .replace('\\', "/");
        if !metadata.is_file()
            || relative != "inference-runtime-manifest.json"
                && !manifest.files.iter().any(|file| file.path == relative)
        {
            return Err("runtime_mismatch: inference runtime contains an unverified file".into());
        }
        actual += 1;
        if actual > 257 {
            return Err("runtime_mismatch: inference inventory exceeds its bound".into());
        }
    }
    if actual != manifest.files.len() + 1 {
        return Err("runtime_mismatch: inference inventory is incomplete".into());
    }
    let mut names = HashSet::new();
    for file in &manifest.files {
        if !valid_inventory_path(directory, &file.path)
            || !names.insert(&file.path)
            || file.size == 0
            || file.size > 2 * 1024 * 1024 * 1024
        {
            return Err("runtime_mismatch: invalid inference runtime file inventory".into());
        }
        let mut stream = artifact::open_regular(&directory.join(&file.path))?;
        if stream.metadata().map_err(|error| error.to_string())?.len() != file.size {
            return Err("runtime_mismatch: inference runtime file size differs".into());
        }
        let mut hash = Sha256::new();
        let mut buffer = [0u8; 65536];
        loop {
            let count = stream
                .read(&mut buffer)
                .map_err(|error| error.to_string())?;
            if count == 0 {
                break;
            }
            hash.update(&buffer[..count]);
        }
        if hex::encode(hash.finalize()) != file.sha256 {
            return Err("runtime_mismatch: inference runtime file hash differs".into());
        }
    }
    Ok(())
}

fn valid_inventory_path(directory: &Path, relative: &str) -> bool {
    if relative.len() > 240 || relative.contains('\\') {
        return false;
    }
    let parts: Vec<_> = relative.split('/').collect();
    if parts.is_empty()
        || parts
            .iter()
            .any(|part| !artifact::valid_name(part, 160, true) || matches!(*part, "." | ".."))
    {
        return false;
    }
    let mut parent = directory.to_path_buf();
    for part in &parts[..parts.len() - 1] {
        parent.push(part);
        let Ok(metadata) = fs::symlink_metadata(&parent) else {
            return false;
        };
        if !metadata.is_dir() || metadata.file_type().is_symlink() {
            return false;
        }
        #[cfg(windows)]
        {
            use std::os::windows::fs::MetadataExt;
            if metadata.file_attributes()
                & windows_sys::Win32::Storage::FileSystem::FILE_ATTRIBUTE_REPARSE_POINT
                != 0
            {
                return false;
            }
        }
    }
    true
}
