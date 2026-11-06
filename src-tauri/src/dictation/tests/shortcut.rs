use crate::dictation::shortcut::{DictationShortcutManager, DEFAULT_DICTATION_SHORTCUT};

#[test]
fn shortcut_manager_initializes_with_default_and_unregistered_state() {
    let manager = DictationShortcutManager::new();
    let status = manager.current_status();
    assert_eq!(status.shortcut, DEFAULT_DICTATION_SHORTCUT);
    assert!(!status.registered);
    assert!(status.error.is_none());
}
