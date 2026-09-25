mod bindings;
mod execute;
mod history;
mod outputs;
mod start;
mod validation;

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct StartCapturedWorkflowRequest {
    pub request_id: String,
    pub workflow_id: String,
    pub expected_version: u32,
    pub space_id: String,
    pub bindings: HashMap<String, Vec<String>>,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct BoundResource {
    pub node_id: String,
    pub kind: String,
    pub data_digest: String,
}

pub(crate) use execute::execute_step;
pub(crate) use history::{list as list_history, CapturedWorkflowRun};
pub(crate) use outputs::{list as list_outputs, CapturedRunOutput};
pub(crate) use start::start;
pub(super) use validation::candidate as validate_candidate;
