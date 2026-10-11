use crate::foundation::app_error::AppError;

impl From<agent_client_protocol::Error> for AppError {
    fn from(error: agent_client_protocol::Error) -> Self {
        use agent_client_protocol::ErrorCode;
        let code = match error.code {
            ErrorCode::InvalidRequest | ErrorCode::InvalidParams => "invalid_request",
            ErrorCode::AuthRequired => "authentication_failed",
            ErrorCode::RequestCancelled => "cancelled",
            ErrorCode::InternalError => "internal",
            _ => "provider_request_failed",
        };
        Self::new(code, error.to_string())
    }
}
