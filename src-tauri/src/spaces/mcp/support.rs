use rmcp::ErrorData as McpError;
use serde::Serialize;

pub(super) fn json_text(value: impl Serialize) -> Result<String, McpError> {
    serde_json::to_string_pretty(&value).map_err(|error| mcp_error(error.to_string()))
}

pub(super) fn mcp_error(error: impl Into<crate::foundation::app_error::AppError>) -> McpError {
    let error = error.into();
    McpError::internal_error(error.message, Some(serde_json::json!({"code": error.code})))
}
