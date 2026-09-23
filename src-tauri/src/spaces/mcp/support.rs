use rmcp::ErrorData as McpError;
use serde::Serialize;

pub(super) fn json_text(value: impl Serialize) -> Result<String, McpError> {
    serde_json::to_string_pretty(&value).map_err(|error| mcp_error(error.to_string()))
}

pub(super) fn mcp_error(error: impl ToString) -> McpError {
    McpError::internal_error(error.to_string(), None)
}
