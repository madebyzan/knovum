mod contracts;
mod deletion;
mod lifecycle;

pub(crate) use contracts::{
    CanvasSelectionMutationResult, DeleteCanvasSelectionRequest, RestoreCanvasSelectionRequest,
    SetImportedCanvasNodesDeletedRequest,
};
pub(crate) use deletion::delete;
pub(crate) use lifecycle::{restore, set_imported_nodes_deleted};

#[cfg(test)]
mod tests;
