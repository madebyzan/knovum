use tauri_plugin_updater::Error;

use crate::app_error::AppError;
use super::contracts::UpdateFailure;

pub(super) fn classify(error: Error) -> (UpdateFailure, AppError) {
    let (failure, detail) = match error {
        Error::Reqwest(error) => (UpdateFailure::Network, error.without_url().to_string()),
        Error::Network(_) => (UpdateFailure::Network, "Update server rejected the request".into()),
        Error::Minisign(_) | Error::Base64(_) | Error::SignatureUtf8(_) =>
            (UpdateFailure::Signature, "Update signature verification failed".into()),
        Error::ReleaseNotFound | Error::Serialization(_) | Error::TargetNotFound(_)
            | Error::TargetsNotFound(_) | Error::Semver(_) =>
            (UpdateFailure::Metadata, "Update manifest is invalid for this platform".into()),
        _ => (UpdateFailure::Installation, "Update operation failed; inspect native installer diagnostics".into()),
    };
    log::error!("Updater {failure:?}: {detail}");
    (failure, AppError::new("update_failed", detail))
}
