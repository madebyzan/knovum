use crate::{artifact, images::Images};
use knovum_inference_contracts::{
    native_manifest::NativeRuntimeManifest,
    runtime::{RuntimeInitializeParams, RuntimeLimits, RuntimeSession, PROTOCOL_VERSION},
};
use sha2::{Digest, Sha256};
use std::{
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
    Transcription(crate::speech::transcription::Transcriber),
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
        initialize_ort(
            Path::new(&params.runtime_manifest_path),
            &params.runtime_digest,
        )?;
        let engine = match params.capability {
            knovum_inference_contracts::runtime::RuntimeCapability::Embedding => {
                Engine::Embedding(Box::new(crate::embedding::Embedding::new(&params)?))
            }
            knovum_inference_contracts::runtime::RuntimeCapability::SpeechTranscription => {
                Engine::Transcription(crate::speech::transcription::Transcriber::new(&params)?)
            }
            knovum_inference_contracts::runtime::RuntimeCapability::SpeechSeparation => {
                Engine::Separation(crate::speech::separation::Separator::new(&params)?)
            }
            knovum_inference_contracts::runtime::RuntimeCapability::SpeechDiarization => {
                Engine::Diarization(crate::speech::diarization::Diarizer::new(&params)?)
            }
            knovum_inference_contracts::runtime::RuntimeCapability::SpeechAlignment => {
                Engine::Alignment(crate::speech::alignment::Aligner::new(&params)?)
            }
            knovum_inference_contracts::runtime::RuntimeCapability::SpeechSynthesis => {
                Engine::Synthesis(crate::speech::synthesis::Synthesizer::new(&params)?)
            }
            _ => Engine::Images(Images::new(&params)?),
        };
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
                limits: RuntimeLimits {
                    max_message_bytes: 1_048_576,
                    max_embedding_texts: 32,
                    max_embedding_tokens: 2048,
                    max_unconsumed_chunk_files: 8,
                    max_unconsumed_chunk_bytes: 16 * 1024 * 1024,
                },
            },
        })
    }
}

pub(crate) fn initialize_ort(path: &Path, expected_digest: &str) -> Result<(), String> {
    #[cfg(windows)]
    if windows_version::OsVersion::current() < windows_version::OsVersion::new(10, 0, 0, 19041) {
        return Err(
            "unsupported_device: native inference requires Windows 10 2004 (build 19041) or later"
                .into(),
        );
    }
    if !path.is_absolute() {
        return Err("runtime_mismatch: native manifest path is relative".into());
    }
    let mut bytes = Vec::new();
    artifact::open_regular(path)?
        .take(1_048_577)
        .read_to_end(&mut bytes)
        .map_err(|error| error.to_string())?;
    if bytes.len() > 1_048_576 || hex::encode(Sha256::digest(&bytes)) != expected_digest {
        return Err("runtime_mismatch: native manifest digest differs".into());
    }
    let manifest: NativeRuntimeManifest =
        serde_json::from_slice(&bytes).map_err(|error| format!("runtime_mismatch: {error}"))?;
    let target = match (std::env::consts::OS, std::env::consts::ARCH) {
        ("macos", "x86_64") => "macos-x64",
        ("macos", "aarch64") => "macos-arm64",
        ("windows", "x86_64") => "windows-x64",
        _ => return Err("unsupported_device: native runtime target is unsupported".into()),
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
        return Err("runtime_mismatch: native runtime manifest is invalid".into());
    }
    let directory = path
        .parent()
        .ok_or("runtime_mismatch: native manifest parent is missing")?;
    artifact::require_directory(directory)?;
    let mut actual = 0;
    for entry in walkdir::WalkDir::new(directory).follow_links(false) {
        let entry = entry.map_err(|error| format!("runtime_mismatch: {error}"))?;
        let metadata = fs::symlink_metadata(entry.path()).map_err(|error| error.to_string())?;
        if metadata.file_type().is_symlink() {
            return Err("runtime_mismatch: native runtime contains a link".into());
        }
        #[cfg(windows)]
        {
            use std::os::windows::fs::MetadataExt;
            if metadata.file_attributes()
                & windows_sys::Win32::Storage::FileSystem::FILE_ATTRIBUTE_REPARSE_POINT
                != 0
            {
                return Err("runtime_mismatch: native runtime contains a reparse point".into());
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
            || relative != "native-runtime-manifest.json"
                && !manifest.files.iter().any(|file| file.path == relative)
        {
            return Err("runtime_mismatch: native runtime contains an unverified file".into());
        }
        actual += 1;
        if actual > 257 {
            return Err("runtime_mismatch: native inventory exceeds its bound".into());
        }
    }
    if actual != manifest.files.len() + 1 {
        return Err("runtime_mismatch: native inventory is incomplete".into());
    }
    let mut names = std::collections::HashSet::new();
    for file in &manifest.files {
        if !valid_inventory_path(directory, &file.path)
            || !names.insert(&file.path)
            || file.size == 0
            || file.size > 2 * 1024 * 1024 * 1024
        {
            return Err("runtime_mismatch: invalid native runtime file inventory".into());
        }
        let mut stream = artifact::open_regular(&directory.join(&file.path))?;
        if stream.metadata().map_err(|error| error.to_string())?.len() != file.size {
            return Err("runtime_mismatch: native runtime file size differs".into());
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
            return Err("runtime_mismatch: native runtime file hash differs".into());
        }
    }
    #[cfg(windows)]
    crate::speech::native::prepare_search_directory(directory)?;
    ort::init_from(directory.join(&manifest.onnx_runtime))
        .map_err(|error| format!("runtime_mismatch: {error}"))?
        .with_telemetry(false)
        .commit();
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
