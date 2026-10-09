use std::fmt;

use crate::local_resources::{
    preparation::RESOURCE_PREPARATION_CANCELLED, verification::HashError,
};

#[derive(Debug)]
pub(crate) enum RuntimeError {
    InvalidPublication(String),
    Cancelled,
    Infrastructure(String),
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPublication(message) => {
                write!(formatter, "Invalid Python runtime: {message}")
            }
            Self::Cancelled => formatter.write_str(RESOURCE_PREPARATION_CANCELLED),
            Self::Infrastructure(message) => formatter.write_str(message),
        }
    }
}

impl std::error::Error for RuntimeError {}

impl From<String> for RuntimeError {
    fn from(message: String) -> Self {
        Self::Infrastructure(message)
    }
}

impl From<std::io::Error> for RuntimeError {
    fn from(error: std::io::Error) -> Self {
        Self::Infrastructure(error.to_string())
    }
}

impl From<HashError> for RuntimeError {
    fn from(error: HashError) -> Self {
        match error {
            HashError::Cancelled => Self::Cancelled,
            HashError::Io(error) => error.into(),
        }
    }
}

impl From<RuntimeError> for String {
    fn from(error: RuntimeError) -> Self {
        error.to_string()
    }
}
