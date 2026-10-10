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
};

pub struct PreparedVoxCpm2Model {
    directory: PathBuf,
    model: super::super::model_catalog::LocalModelId,
    _models: PreparedModels,
}

pub struct PreparedDiarizationModel {
    directory: PathBuf,
    variant: catalog::DiarizationVariant,
    _models: PreparedModels,
}

impl PreparedDiarizationModel {
    pub fn from_models(
        models: PreparedModels,
        variant: catalog::DiarizationVariant,
    ) -> Result<Self, String> {
        let directory = models.directory(variant.models())?;
        Ok(Self {
            directory,
            variant,
            _models: models,
        })
    }
    pub fn directory(&self) -> &Path {
        &self.directory
    }
    pub(crate) fn spec_id(&self) -> &'static str {
        self.variant.spec_id()
    }
}

impl PreparedVoxCpm2Model {
    pub fn from_models(
        models: PreparedModels,
        model: super::super::model_catalog::LocalModelId,
    ) -> Result<Self, String> {
        let directory = models.directory(model.voxcpm2_models()?)?;
        Ok(Self {
            directory,
            model,
            _models: models,
        })
    }

    pub fn directory(&self) -> &Path {
        &self.directory
    }

    pub(crate) fn spec_id(&self) -> Result<&'static str, String> {
        self.model.voxcpm2_spec()
    }
}

#[derive(Clone)]
pub struct PreparedModels {
    paths: Arc<HashMap<ModelKind, PathBuf>>,
    _usage: Arc<Vec<OwnedRwLockReadGuard<()>>>,
}

impl PreparedModels {
    pub(crate) fn directory(&self, kinds: &[ModelKind]) -> Result<PathBuf, String> {
        let anchor = self.path(kinds[0])?;
        let directory = anchor
            .parent()
            .ok_or("Model snapshot directory is unavailable")?
            .to_owned();
        for kind in kinds {
            if self.path(*kind)? != directory.join(catalog::descriptor(*kind).file) {
                return Err("runtime_mismatch: model snapshot is not co-located".into());
            }
        }
        Ok(directory)
    }
    pub(crate) fn transcription_model(
        &self,
    ) -> Result<super::super::model_catalog::LocalModelId, String> {
        use super::super::{
            model_catalog::LocalModelId as M,
            models::{catalog::WhisperVariant as V, snapshots::SnapshotKind as S},
        };
        let snapshots = self
            .paths
            .keys()
            .map(|kind| catalog::descriptor(*kind).snapshot)
            .filter(|snapshot| matches!(snapshot, S::FasterWhisper | S::MlxWhisper | S::Whisper(_)))
            .collect::<std::collections::HashSet<_>>();
        if snapshots.len() != 1 {
            return Err(
                "runtime_mismatch: ASR requires exactly one prepared model snapshot".into(),
            );
        }
        match snapshots.into_iter().next().unwrap() {
            S::FasterWhisper | S::MlxWhisper => Ok(M::WhisperSmall),
            S::Whisper(V::FasterTiny | V::MlxTiny) => Ok(M::WhisperTiny),
            S::Whisper(V::FasterLargeV3 | V::MlxLargeV3) => Ok(M::WhisperLargeV3),
            _ => unreachable!("Only ASR snapshots remain after filtering"),
        }
    }

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

pub(crate) fn resource_lock(
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
