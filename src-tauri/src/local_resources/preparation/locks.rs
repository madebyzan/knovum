use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{Arc, Mutex, OnceLock},
};

use tauri::{AppHandle, Runtime};
use tokio::sync::{OwnedRwLockReadGuard, RwLock};
use tokio_util::sync::CancellationToken;

use super::RESOURCE_PREPARATION_CANCELLED;
use crate::local_resources::models::{
    self,
    catalog::{self, ModelKind},
    diarization::VerifiedDiarizationDirectory,
    kokoro::VerifiedKokoroDirectory,
};

fn kokoro_lock() -> Arc<RwLock<()>> {
    static LOCK: OnceLock<Arc<RwLock<()>>> = OnceLock::new();
    Arc::clone(LOCK.get_or_init(|| Arc::new(RwLock::new(()))))
}

fn diarization_lock() -> Arc<RwLock<()>> {
    static LOCK: OnceLock<Arc<RwLock<()>>> = OnceLock::new();
    Arc::clone(LOCK.get_or_init(|| Arc::new(RwLock::new(()))))
}

pub(crate) async fn acquire_diarization_installation() -> tokio::sync::OwnedRwLockWriteGuard<()> {
    diarization_lock().write_owned().await
}

pub(super) async fn acquire_kokoro_installation(
    cancellation: &CancellationToken,
) -> Result<tokio::sync::OwnedRwLockWriteGuard<()>, String> {
    tokio::select! {
        biased;
        _ = cancellation.cancelled() => Err(RESOURCE_PREPARATION_CANCELLED.to_string()),
        guard = kokoro_lock().write_owned() => Ok(guard),
    }
}

pub struct PreparedKokoroModel {
    directory: VerifiedKokoroDirectory,
    _usage: OwnedRwLockReadGuard<()>,
}

pub struct PreparedVoxCpm2Model {
    directory: PathBuf,
    _models: PreparedModels,
}

pub struct PreparedDiarizationModel {
    directory: VerifiedDiarizationDirectory,
    _usage: OwnedRwLockReadGuard<()>,
}

impl PreparedDiarizationModel {
    pub fn directory(&self) -> &Path {
        self.directory.as_path()
    }
}

pub async fn acquire_verified_diarization<R: Runtime>(
    app: &AppHandle<R>,
    cancellation: &CancellationToken,
) -> Result<PreparedDiarizationModel, String> {
    let usage = tokio::select! {
        biased;
        _ = cancellation.cancelled() => return Err(RESOURCE_PREPARATION_CANCELLED.into()),
        guard = diarization_lock().read_owned() => guard,
    };
    let app = app.clone();
    let directory = crate::blocking::run(move || {
        crate::local_resources::models::diarization::require_verified_directory(&app)
    })
    .await?;
    Ok(PreparedDiarizationModel {
        directory,
        _usage: usage,
    })
}

impl PreparedVoxCpm2Model {
    pub fn from_models(models: PreparedModels) -> Result<Self, String> {
        let anchor = models.path(catalog::VOXCPM2_MODELS[0])?;
        let directory = anchor
            .parent()
            .ok_or_else(|| "VoxCPM2 model directory is unavailable".to_string())?
            .to_owned();
        for kind in catalog::VOXCPM2_MODELS {
            let actual = models.path(*kind)?;
            let expected = directory.join(catalog::descriptor(*kind).file);
            if actual != expected {
                return Err("VoxCPM2 model snapshot is not co-located".to_string());
            }
        }
        Ok(Self {
            directory,
            _models: models,
        })
    }

    pub fn directory(&self) -> &Path {
        &self.directory
    }
}

impl PreparedKokoroModel {
    pub fn directory(&self) -> &VerifiedKokoroDirectory {
        &self.directory
    }
}

pub async fn acquire_verified_kokoro<R: Runtime>(
    app: &AppHandle<R>,
    cancellation: &CancellationToken,
) -> Result<PreparedKokoroModel, String> {
    let usage = tokio::select! {
        biased;
        _ = cancellation.cancelled() => return Err(RESOURCE_PREPARATION_CANCELLED.to_string()),
        guard = kokoro_lock().read_owned() => guard,
    };
    let app = app.clone();
    let cancellation = cancellation.clone();
    let directory = crate::blocking::run(move || {
        models::kokoro::require_verified_directory(&app, &cancellation)
    })
    .await?;
    Ok(PreparedKokoroModel {
        directory,
        _usage: usage,
    })
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(super) struct ModelIdentity {
    directory: &'static str,
    file: &'static str,
    revision: &'static str,
    sha256: &'static str,
}

impl ModelIdentity {
    pub(super) fn for_kind(kind: ModelKind) -> Self {
        let descriptor = catalog::descriptor(kind);
        Self {
            directory: descriptor.directory,
            file: descriptor.file,
            revision: descriptor.revision,
            sha256: descriptor.sha256,
        }
    }
}

struct PreparedModel {
    path: PathBuf,
    _usage: OwnedRwLockReadGuard<()>,
}

#[derive(Clone)]
pub struct PreparedModels {
    models: Arc<HashMap<ModelKind, PreparedModel>>,
}

impl PreparedModels {
    pub fn path(&self, kind: ModelKind) -> Result<&Path, String> {
        self.models
            .get(&kind)
            .map(|model| model.path.as_path())
            .ok_or_else(|| "Prepared model was not requested".to_string())
    }
}

pub async fn acquire_verified_models<R: Runtime>(
    app: &AppHandle<R>,
    kinds: &[ModelKind],
    request_cancellation: &CancellationToken,
) -> Result<PreparedModels, String> {
    let mut ordered = kinds.to_vec();
    ordered.sort_by_key(|kind| model_order(*kind));
    if ordered.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err("Model usage contains duplicate model parts".to_string());
    }
    let mut prepared = HashMap::with_capacity(kinds.len());
    for kind in ordered {
        let lock = resource_lock(ModelIdentity::for_kind(kind))?;
        let usage = tokio::select! {
            biased;
            _ = request_cancellation.cancelled() => {
                return Err(RESOURCE_PREPARATION_CANCELLED.to_string());
            }
            guard = lock.read_owned() => guard,
        };
        let path = models::require_verified_path(app, catalog::descriptor(kind))?;
        prepared.insert(
            kind,
            PreparedModel {
                path,
                _usage: usage,
            },
        );
    }
    Ok(PreparedModels {
        models: Arc::new(prepared),
    })
}

const fn model_order(kind: ModelKind) -> u8 {
    match kind {
        ModelKind::FasterWhisperConfig => 0,
        ModelKind::FasterWhisperWeights => 1,
        ModelKind::FasterWhisperTokenizer => 2,
        ModelKind::FasterWhisperVocabulary => 3,
        ModelKind::MlxWhisperConfig => 4,
        ModelKind::MlxWhisperWeights => 5,
        ModelKind::ImageSegmentation => 6,
        ModelKind::TextEmbeddingConfig => 7,
        ModelKind::TextEmbeddingModules => 8,
        ModelKind::TextEmbeddingPoolingConfig => 9,
        ModelKind::TextEmbeddingSentenceTransformersConfig => 10,
        ModelKind::TextEmbeddingTokenizerConfig => 11,
        ModelKind::TextEmbeddingTokenizer => 12,
        ModelKind::TextEmbeddingWeights => 13,
        ModelKind::VoxCpm2AudioVae => 14,
        ModelKind::VoxCpm2Config => 15,
        ModelKind::VoxCpm2Weights => 16,
        ModelKind::VoxCpm2SpecialTokens => 17,
        ModelKind::VoxCpm2TokenizerImplementation => 18,
        ModelKind::VoxCpm2Tokenizer => 19,
        ModelKind::VoxCpm2TokenizerConfig => 20,
        ModelKind::Qwen3ForcedAlignerChatTemplate => 21,
        ModelKind::Qwen3ForcedAlignerConfig => 22,
        ModelKind::Qwen3ForcedAlignerGenerationConfig => 23,
        ModelKind::Qwen3ForcedAlignerMerges => 24,
        ModelKind::Qwen3ForcedAlignerWeights => 25,
        ModelKind::Qwen3ForcedAlignerPreprocessorConfig => 26,
        ModelKind::Qwen3ForcedAlignerTokenizerConfig => 27,
        ModelKind::Qwen3ForcedAlignerVocabulary => 28,
        ModelKind::SpleeterAccompaniment => 29,
        ModelKind::SpleeterVocals => 30,
    }
}

pub(super) fn resource_lock(identity: ModelIdentity) -> Result<Arc<RwLock<()>>, String> {
    static LOCKS: OnceLock<Mutex<HashMap<ModelIdentity, Arc<RwLock<()>>>>> = OnceLock::new();
    let locks = LOCKS.get_or_init(|| Mutex::new(HashMap::new()));
    let mut locks = locks
        .lock()
        .map_err(|_| "Model resource lock registry poisoned".to_string())?;
    Ok(Arc::clone(
        locks
            .entry(identity)
            .or_insert_with(|| Arc::new(RwLock::new(()))),
    ))
}
