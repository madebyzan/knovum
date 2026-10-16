//! Layers that were discovered and written to disk but not yet published into output nodes.
//! Splitting runs in two phases because the layer count is discovered rather than requested: the
//! caller can only create the output nodes once discovery has reported how many there are.

use std::{
    collections::HashMap,
    sync::{Arc, Mutex, OnceLock},
};

use super::{
    assets::{self, PreparedLayer},
    registry::Generation,
};
use crate::foundation::job_registry::JobLease;

/// Staged plans hold files on disk, so the store is bounded and the oldest plan is discarded
/// once the bound is reached.
const MAX_PLANS: usize = 4;

pub(super) struct StagedPlan {
    pub(super) layers: Vec<PreparedLayer>,
    pub(super) space_id: String,
    pub(super) source_node_id: String,
    pub(super) source_revision: String,
    pub(super) generation: Arc<Generation>,
    /// Held so the split stays cancellable until its layers are published.
    pub(super) lease: JobLease<Arc<Generation>>,
}

type Store = Mutex<(Vec<String>, HashMap<String, StagedPlan>)>;

fn store() -> &'static Store {
    static STORE: OnceLock<Store> = OnceLock::new();
    STORE.get_or_init(|| Mutex::new((Vec::new(), HashMap::new())))
}

pub(super) fn stage(plan_id: &str, plan: StagedPlan) -> Result<(), String> {
    let mut store = store().lock().map_err(|_| "Image layer plans poisoned")?;
    let (order, plans) = &mut *store;
    if plans.insert(plan_id.to_owned(), plan).is_none() {
        order.push(plan_id.to_owned());
    }
    while order.len() > MAX_PLANS {
        let discarded = order.remove(0);
        if let Some(plan) = plans.remove(&discarded) {
            assets::remove(&plan.layers);
        }
    }
    Ok(())
}

pub(super) fn take(plan_id: &str) -> Result<StagedPlan, String> {
    let mut store = store().lock().map_err(|_| "Image layer plans poisoned")?;
    let (order, plans) = &mut *store;
    order.retain(|staged| staged != plan_id);
    plans
        .remove(plan_id)
        .ok_or_else(|| "Image layer plan is no longer available; split the image again".into())
}

/// Drops a plan and the files it staged. Used when a split is cancelled after discovery.
pub(super) fn discard(plan_id: &str) {
    if let Ok(plan) = take(plan_id) {
        assets::remove(&plan.layers);
    }
}
