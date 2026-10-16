mod appearance;
mod clipboard;
pub(crate) mod commands;
mod content_height_read_set;
mod contracts;
pub(crate) mod creation;
pub(crate) mod deletion;
mod deletion_transaction;
pub(crate) mod edit_contract;
pub(crate) mod editor_protection;
mod geometry;
pub(crate) mod history;
mod node_reference_remap;
mod placement;
mod planner;
pub(crate) mod portal_integrity;
pub(crate) mod queries;
pub(crate) mod receipts;
pub(crate) mod reclamation;
pub(crate) mod repository;
pub(crate) mod search;
mod validation;

pub(crate) use contracts::{
    LocalTransform, Point, RelationColor, RelationStyle, Size, SizingMode, StructureLayout,
    StructureNode,
};
pub(crate) use edit_contract::{ExecuteSpaceEditRequest, SpaceEditSnapshotResponse};
pub(crate) use placement::{place_new_content, CONTENT_GAP};
pub(crate) use repository::initialize_empty_in_transaction;
pub(crate) use validation::{validate_content_coverage, validate_space_snapshot};

#[cfg(test)]
mod tests;

pub(crate) mod transfer;
