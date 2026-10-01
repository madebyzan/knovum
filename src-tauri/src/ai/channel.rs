use crate::app_error::{AppError, AppResult};

pub(crate) fn send_channel_event<T: serde::Serialize>(
    channel: &tauri::ipc::Channel<T>,
    event: T,
) -> AppResult<()> {
    channel
        .send(event)
        .map_err(|error| AppError::internal(error.to_string()))
}
