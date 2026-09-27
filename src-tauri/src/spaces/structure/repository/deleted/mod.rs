use super::super::{StructureLayout, StructureNode};

pub(crate) struct RestoredStructure {
    pub batch_ids: Vec<String>,
    pub structure: Vec<StructureNode>,
    pub layouts: Vec<StructureLayout>,
    pub relation_styles: Vec<super::super::RelationStyle>,
    pub relations: Vec<RestoredRelation>,
}

pub(crate) struct RestoredRelation {
    pub id: String,
    pub status: String,
    pub status_before_invalidation: Option<String>,
}

mod merge;
mod restore;
mod root;
mod store;

pub(crate) use merge::merge_restored_snapshot;
pub(crate) use restore::{mark_restored, prepare_restore_batch, remove_tombstones};
pub(crate) use root::prepare_restore_root;
pub(crate) use store::{store_deletion_in_transaction, DeletionBatch};
