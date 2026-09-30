use serde::Serialize;

pub type AiResult<T> = Result<T, AiError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ts_rs::TS, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AiErrorCode {
    InvalidRequest,
    UnsupportedProvider,
    NotConfigured,
    AgentUnavailable,
    AuthenticationFailed,
    CapabilityNotSupported,
    ProviderRequestFailed,
    InvalidResponse,
    EmptyResponse,
    Cancelled,
    Internal,
}

#[derive(Debug, Clone, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct AiError {
    pub code: AiErrorCode,
    pub message: String,
}

impl AiError {
    pub fn new(code: AiErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }

    /// Carry machine-readable AI errors through feature commands that use CommandError.
    pub fn task_message(&self) -> String {
        let code = serde_json::to_value(self.code).expect("AI error code is serializable");
        format!(
            "ai-{}: {}",
            code.as_str()
                .expect("AI error code is a string")
                .replace('_', "-"),
            self.message
        )
    }

    pub fn invalid(message: impl Into<String>) -> Self {
        Self::new(AiErrorCode::InvalidRequest, message)
    }

    pub fn not_configured(message: impl Into<String>) -> Self {
        Self::new(AiErrorCode::NotConfigured, message)
    }

    pub fn unsupported(message: impl Into<String>) -> Self {
        Self::new(AiErrorCode::UnsupportedProvider, message)
    }

    pub fn capability_not_supported(provider: &str, capability: &str) -> Self {
        Self::new(
            AiErrorCode::CapabilityNotSupported,
            format!("Provider '{provider}' does not support '{capability}'"),
        )
    }

    pub fn cancelled(message: impl Into<String>) -> Self {
        Self::new(AiErrorCode::Cancelled, message)
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(AiErrorCode::Internal, message)
    }
}

impl From<String> for AiError {
    fn from(message: String) -> Self {
        Self::new(AiErrorCode::Internal, message)
    }
}

impl From<agent_client_protocol::Error> for AiError {
    fn from(error: agent_client_protocol::Error) -> Self {
        use agent_client_protocol::ErrorCode;

        let code = match error.code {
            ErrorCode::InvalidRequest | ErrorCode::InvalidParams => AiErrorCode::InvalidRequest,
            ErrorCode::AuthRequired => AiErrorCode::AuthenticationFailed,
            ErrorCode::RequestCancelled => AiErrorCode::Cancelled,
            ErrorCode::InternalError => AiErrorCode::Internal,
            _ => AiErrorCode::ProviderRequestFailed,
        };
        Self::new(code, error.to_string())
    }
}

impl std::fmt::Display for AiError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for AiError {}

pub(crate) fn send_channel_event<T: serde::Serialize>(
    channel: &tauri::ipc::Channel<T>,
    event: T,
) -> AiResult<()> {
    channel
        .send(event)
        .map_err(|error| AiError::new(AiErrorCode::Internal, error.to_string()))
}
