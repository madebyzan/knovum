mod clipboard;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

use crate::dictation_contracts::InsertionOutcome;

#[cfg(target_os = "macos")]
pub(crate) use macos::TargetApplication;
#[cfg(target_os = "windows")]
pub(crate) use windows::TargetApplication;

pub(crate) struct InsertionDecision {
    pub(crate) outcome: InsertionOutcome,
    pub(crate) message: Option<String>,
}

impl InsertionDecision {
    fn inserted() -> Self {
        Self {
            outcome: InsertionOutcome::Inserted,
            message: None,
        }
    }

    fn rejected(outcome: InsertionOutcome, message: impl Into<String>) -> Self {
        Self {
            outcome,
            message: Some(message.into()),
        }
    }
}

pub(super) struct PasteError {
    outcome: InsertionOutcome,
    message: String,
}

impl PasteError {
    #[cfg(any(target_os = "windows", test))]
    pub(super) fn permission_denied(message: impl Into<String>) -> Self {
        Self {
            outcome: InsertionOutcome::PermissionDenied,
            message: message.into(),
        }
    }

    #[cfg(any(target_os = "macos", test))]
    pub(super) fn failed(message: impl Into<String>) -> Self {
        Self {
            outcome: InsertionOutcome::Failed,
            message: message.into(),
        }
    }

    fn outcome(&self) -> InsertionOutcome {
        self.outcome
    }

    fn message(&self) -> &str {
        &self.message
    }
}

#[cfg(any(target_os = "macos", target_os = "windows"))]
pub(crate) fn capture_target() -> Result<TargetApplication, String> {
    platform::capture_target()
}

#[cfg(any(target_os = "macos", target_os = "windows"))]
pub(crate) fn insert(target: &TargetApplication, text: &str) -> InsertionDecision {
    if text.trim().is_empty() {
        return InsertionDecision::rejected(
            InsertionOutcome::NotAttempted,
            "The transcription did not contain insertable text",
        );
    }
    if !platform::has_permission() {
        return InsertionDecision::rejected(
            InsertionOutcome::PermissionDenied,
            platform::permission_message(),
        );
    }
    if !platform::is_focused(target) {
        return InsertionDecision::rejected(
            InsertionOutcome::FocusChanged,
            "The target application changed before dictation completed",
        );
    }
    let mut transaction = match clipboard::ClipboardTransaction::begin(text) {
        Ok(value) => value,
        Err(error) => return InsertionDecision::rejected(InsertionOutcome::Failed, error),
    };
    if !platform::is_focused(target) {
        transaction.restore_if_unchanged();
        return InsertionDecision::rejected(
            InsertionOutcome::FocusChanged,
            "The target application changed before text insertion",
        );
    }
    if let Err(error) = platform::paste() {
        transaction.restore_if_unchanged();
        return InsertionDecision::rejected(error.outcome(), error.message());
    }
    std::thread::sleep(std::time::Duration::from_millis(180));
    transaction.restore_if_unchanged();
    InsertionDecision::inserted()
}

pub(crate) fn copy_text(text: String) -> Result<(), String> {
    clipboard::write_text(text)
}

#[cfg(target_os = "macos")]
use macos as platform;
#[cfg(target_os = "windows")]
use windows as platform;

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
compile_error!("dictation insertion is only compiled for supported desktop targets");

#[cfg(test)]
mod tests;
