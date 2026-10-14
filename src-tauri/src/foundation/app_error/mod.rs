use serde::Serialize;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Clone, Debug, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct AppError {
    pub code: String,
    pub message: String,
}

impl AppError {
    pub fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
        }
    }

    pub fn invalid(message: impl Into<String>) -> Self {
        Self::new("invalid_request", message)
    }

    pub fn not_configured(message: impl Into<String>) -> Self {
        Self::new("not_configured", message)
    }

    pub fn unsupported(message: impl Into<String>) -> Self {
        Self::new("unsupported_provider", message)
    }

    pub fn capability_not_supported(provider: &str, capability: &str) -> Self {
        Self::new(
            "capability_not_supported",
            format!("Provider '{provider}' does not support '{capability}'"),
        )
    }

    pub fn cancelled(message: impl Into<String>) -> Self {
        Self::new("cancelled", message)
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::new("internal", message)
    }
}

impl From<String> for AppError {
    fn from(message: String) -> Self {
        Self::new("operation_failed", message)
    }
}

impl From<&str> for AppError {
    fn from(message: &str) -> Self {
        message.to_owned().into()
    }
}

impl From<AppError> for String {
    fn from(error: AppError) -> Self {
        error.message
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for AppError {}

#[cfg(test)]
mod tests;
