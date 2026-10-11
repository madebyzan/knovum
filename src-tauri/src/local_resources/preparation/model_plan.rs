use std::collections::HashSet;

use tauri::{AppHandle, Runtime};

use crate::local_resources::models::{
    self,
    catalog::{self, ModelKind},
};

pub(super) struct ModelPlan {
    pub(super) missing: Vec<ModelKind>,
    pub(super) download_bytes: Option<u64>,
}

pub(super) fn inspect<R: Runtime>(
    app: &AppHandle<R>,
    kinds: &[ModelKind],
) -> Result<ModelPlan, String> {
    let mut snapshots = HashSet::new();
    let mut missing = Vec::new();
    for &kind in kinds {
        let descriptor = catalog::descriptor(kind);
        let status = models::status(app, descriptor)?;
        if !status.verified && snapshots.insert(descriptor.snapshot) {
            missing.push(kind);
        }
    }
    let model_bytes = missing
        .iter()
        .map(|kind| catalog::descriptor(*kind).snapshot.download_bytes())
        .sum::<u64>();
    Ok(ModelPlan {
        missing,
        download_bytes: Some(model_bytes),
    })
}
