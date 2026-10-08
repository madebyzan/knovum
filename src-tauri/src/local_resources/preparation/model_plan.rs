use std::collections::{BTreeSet, HashSet};

use tauri::{AppHandle, Runtime};

use crate::local_resources::{
    models::{
        self,
        catalog::{self, ModelKind},
    },
    runtimes::python::{self, RuntimeProfile},
};

pub(super) struct ModelPlan {
    pub(super) runtimes: Vec<(RuntimeProfile, Option<u64>)>,
    pub(super) missing: Vec<ModelKind>,
    pub(super) download_bytes: Option<u64>,
}

pub(super) fn inspect<R: Runtime>(
    app: &AppHandle<R>,
    kinds: &[ModelKind],
) -> Result<ModelPlan, String> {
    let mut snapshots = HashSet::new();
    let mut missing = Vec::new();
    let mut profiles = kinds
        .iter()
        .map(|kind| catalog::descriptor(*kind).snapshot.runtime_profile())
        .collect::<BTreeSet<_>>();
    for &kind in kinds {
        let descriptor = catalog::descriptor(kind);
        let status = models::status(app, descriptor)?;
        if !status.verified && snapshots.insert(descriptor.snapshot) {
            missing.push(kind);
        }
    }
    if missing.iter().any(|kind| {
        super::super::models::sources::for_snapshot(catalog::descriptor(*kind).snapshot).is_some()
    }) {
        profiles.insert(RuntimeProfile::Resources);
    }
    let runtimes = profiles
        .into_iter()
        .map(|profile| Ok((profile, python::download_size(app, profile)?)))
        .collect::<Result<Vec<_>, String>>()?;
    let model_bytes = missing
        .iter()
        .map(|kind| catalog::descriptor(*kind).snapshot.total_bytes())
        .sum::<u64>();
    let download_bytes = runtimes.iter().try_fold(model_bytes, |total, (_, size)| {
        size.map(|bytes| total + bytes)
    });
    Ok(ModelPlan {
        runtimes,
        missing,
        download_bytes,
    })
}
