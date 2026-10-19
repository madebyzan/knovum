//! Step configurations for the speech project workflow.
//!
//! Planning (`run_start`) and execution (`run_execution`) share these types, so
//! a step can never be written with a shape its executor refuses to read.

use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::spaces::db;

/// Steps that only need to re-check the revision they were planned against.
/// Used by audiobook loudness measurement and audiobook encoding, whose project
/// type is already pinned by the step kind.
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct RevisionStep {
    pub(super) project_id: String,
    pub(super) revision_id: String,
    pub(super) revision: i64,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct SegmentStep {
    pub(super) project_id: String,
    pub(super) revision_id: String,
    pub(super) segment_id: String,
    pub(super) ordinal: i64,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct ChapterStep {
    pub(super) project_id: String,
    pub(super) revision_id: String,
    pub(super) revision: i64,
    pub(super) chapter_key: String,
    pub(super) chapter_index: usize,
}

/// Assembly serves every non-audiobook project type, so it carries the type it
/// was planned for and refuses a project that changed underneath it.
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct AssemblyStep {
    pub(super) project_id: String,
    pub(super) revision_id: String,
    pub(super) revision: i64,
    pub(super) project_type: String,
}

pub(super) fn parse<C: DeserializeOwned>(configuration: &serde_json::Value) -> Result<C, String> {
    serde_json::from_value(configuration.clone()).map_err(db::json_error)
}
