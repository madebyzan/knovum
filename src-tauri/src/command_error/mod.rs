use serde::Serialize;

#[derive(Clone, Debug, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CommandError {
    pub code: String,
    pub message: String,
}

impl CommandError {
    pub(crate) fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
        }
    }
}

impl From<String> for CommandError {
    fn from(message: String) -> Self {
        let candidate = message
            .split_once(':')
            .map_or(message.as_str(), |(code, _)| code);
        let coded = candidate.contains('-')
            && candidate.len() <= 80
            && candidate
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-');
        let code = if coded {
            candidate.replace('-', "_")
        } else {
            "operation_failed".into()
        };
        Self { code, message }
    }
}
impl From<&str> for CommandError {
    fn from(message: &str) -> Self {
        message.to_owned().into()
    }
}
impl std::fmt::Display for CommandError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}
impl std::error::Error for CommandError {}

#[cfg(test)]
mod tests;
