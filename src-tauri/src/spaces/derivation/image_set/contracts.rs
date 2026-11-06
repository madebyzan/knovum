use serde::Deserialize;
use serde_json::Value;

/// A derivation that publishes a set of images whose size the source decides: the layers a picture
/// separates into, the stickers a sheet carries. Everything that differs between such sets is
/// stated here once; reserving, opening, settling and failing the set is shared.
pub(crate) struct ImageSet {
    /// The Action the set's derivations record.
    pub(crate) action_kind: &'static str,
    /// The engine the set's derivations record.
    pub(crate) engine: &'static str,
    /// Why every member derives from the source. The Run's first output and the members reserved
    /// beside it carry the same lineage.
    pub(crate) lineage: &'static str,
    /// What one member is called in its node id, derivation id and title.
    pub(crate) member: &'static str,
    /// Inclusive bounds on how many members describe such a set.
    pub(crate) members: std::ops::RangeInclusive<usize>,
    /// Why a source whose discovered count falls outside `members` does not settle.
    pub(crate) unsupported: &'static str,
}

/// One set request. `id` is the Run that owns it, so the outputs and their derivation rows are
/// named after it and the set settles against a single authority. `output_node_id` is the node the
/// Run reserved before discovery ran, which the first member publishes into.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ImageSetRequest {
    pub id: String,
    pub space_id: String,
    pub output_node_id: String,
    pub source_node_id: String,
    pub source_revision: String,
    pub source_edit: Value,
}

pub fn validate(request: &ImageSetRequest) -> Result<(), String> {
    identity(&request.id, "Image set Run")?;
    identity(&request.space_id, "Space")?;
    identity(&request.output_node_id, "Image set output node")?;
    identity(&request.source_node_id, "Image set source node")?;
    identity(&request.source_revision, "Image set source revision")?;
    if !(request.source_edit.is_null() || request.source_edit.is_object()) {
        return Err("Image set source edit must be an object or null".into());
    }
    Ok(())
}

impl ImageSet {
    /// Rejects a discovered count that no longer describes this kind of set.
    pub(crate) fn validate_count(&self, count: usize) -> Result<(), String> {
        if !self.members.contains(&count) {
            return Err(self.unsupported.into());
        }
        Ok(())
    }

    /// Identity of one member node. The first member publishes into the node the Run reserved
    /// before it knew how many members there would be; the rest are named after the Run, so a
    /// replayed settlement cannot publish into a node that belongs to another set.
    pub(crate) fn node_id(&self, request: &ImageSetRequest, index: usize) -> String {
        match index {
            0 => request.output_node_id.clone(),
            _ => format!("{}-{}-{index}", request.id, self.member),
        }
    }

    pub(crate) fn derivation_id(&self, run_id: &str, index: usize) -> String {
        format!("{run_id}:{}-{index}", self.member)
    }
}

fn identity(value: &str, label: &str) -> Result<(), String> {
    if value.trim().is_empty() {
        return Err(format!("{label} identity is required"));
    }
    if value.len() > 128 {
        return Err(format!("{label} identity exceeds its limit"));
    }
    Ok(())
}

#[cfg(test)]
#[path = "tests/contracts.rs"]
mod tests;
