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

#[derive(Clone)]
pub struct PreparedModels {
    paths: Arc<HashMap<ModelKind, PathBuf>>,
    _usage: Arc<Vec<OwnedRwLockReadGuard<()>>>,
}

impl PreparedModels {
    pub fn path(&self, kind: ModelKind) -> Result<&Path, String> {
        self.paths
            .get(&kind)
            .map(PathBuf::as_path)
            .ok_or_else(|| "Prepared model was not requested".to_string())
    }
}

pub async fn acquire_verified_models<R: Runtime>(
    app: &AppHandle<R>,
    kinds: &[ModelKind],
    request_cancellation: &CancellationToken,
) -> Result<PreparedModels, String> {
    let mut snapshots = kinds
        .iter()
        .map(|kind| catalog::descriptor(*kind).snapshot)
        .collect::<Vec<_>>();
    snapshots.sort();
    snapshots.dedup();
    let mut usage = Vec::with_capacity(snapshots.len());
    for snapshot in snapshots {
        let lock = resource_lock(snapshot)?;
        usage.push(tokio::select! {
            biased;
            _ = request_cancellation.cancelled() => return Err(RESOURCE_PREPARATION_CANCELLED.to_string()),
            guard = lock.read_owned() => guard,
        });
    }
    let paths = kinds
        .iter()
        .map(|kind| {
            Ok((
                *kind,
                models::require_verified_path(app, catalog::descriptor(*kind))?,
            ))
        })
        .collect::<Result<HashMap<_, _>, String>>()?;
    Ok(PreparedModels {
        paths: Arc::new(paths),
        _usage: Arc::new(usage),
    })
}

pub(super) fn resource_lock(
    identity: models::snapshots::SnapshotKind,
) -> Result<Arc<RwLock<()>>, String> {
    static LOCKS: OnceLock<Mutex<HashMap<models::snapshots::SnapshotKind, Arc<RwLock<()>>>>> =
        OnceLock::new();
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
