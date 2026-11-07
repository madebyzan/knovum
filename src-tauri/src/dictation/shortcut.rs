use std::sync::RwLock;
use tauri::{AppHandle, Runtime};
use tauri_plugin_global_shortcut::GlobalShortcutExt;

use super::contracts::DictationShortcutStatus;

pub(crate) const DEFAULT_DICTATION_SHORTCUT: &str = "CommandOrControl+Shift+Space";

pub(crate) struct DictationShortcutManager {
    status: RwLock<DictationShortcutStatus>,
}

impl DictationShortcutManager {
    pub(crate) fn new() -> Self {
        Self {
            status: RwLock::new(DictationShortcutStatus {
                shortcut: DEFAULT_DICTATION_SHORTCUT.to_string(),
                registered: false,
                error: None,
            }),
        }
    }

    pub(crate) fn register_shortcut<R: Runtime>(
        &self,
        app: &AppHandle<R>,
        shortcut: &str,
    ) -> DictationShortcutStatus {
        let global = app.global_shortcut();
        let mut state = self.status.write().unwrap();
        if state.registered {
            let _ = global.unregister(state.shortcut.as_str());
        }

        let trimmed = shortcut.trim();
        match global.register(trimmed) {
            Ok(()) => {
                log::info!("Registered global dictation shortcut '{trimmed}'");
                *state = DictationShortcutStatus {
                    shortcut: trimmed.to_string(),
                    registered: true,
                    error: None,
                };
            }
            Err(error) => {
                log::warn!(
                    "Global dictation shortcut '{trimmed}' could not be registered: {error}"
                );
                *state = DictationShortcutStatus {
                    shortcut: trimmed.to_string(),
                    registered: false,
                    error: Some(error.to_string()),
                };
            }
        }
        state.clone()
    }

    pub(crate) fn current_status(&self) -> DictationShortcutStatus {
        self.status.read().unwrap().clone()
    }
}

#[cfg(test)]
#[path = "tests/shortcut.rs"]
mod tests;
