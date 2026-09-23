use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowDiagnostic {
    pub code: &'static str,
    pub message: String,
    pub node_id: Option<String>,
    pub edge_id: Option<String>,
}

pub(super) fn global_diagnostic(code: &'static str, message: &str) -> WorkflowDiagnostic {
    WorkflowDiagnostic {
        code,
        message: message.to_owned(),
        node_id: None,
        edge_id: None,
    }
}

pub(super) fn node_diagnostic(
    code: &'static str,
    message: &str,
    node_id: &str,
) -> WorkflowDiagnostic {
    WorkflowDiagnostic {
        code,
        message: message.to_owned(),
        node_id: Some(node_id.to_owned()),
        edge_id: None,
    }
}

pub(super) fn edge_diagnostic(
    code: &'static str,
    message: &str,
    edge_id: &str,
) -> WorkflowDiagnostic {
    WorkflowDiagnostic {
        code,
        message: message.to_owned(),
        node_id: None,
        edge_id: Some(edge_id.to_owned()),
    }
}
