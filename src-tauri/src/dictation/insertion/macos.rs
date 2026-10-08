use core_graphics::{
    event::{CGEvent, CGEventFlags, CGEventTapLocation, KeyCode},
    event_source::{CGEventSource, CGEventSourceStateID},
};
use objc2_app_kit::NSWorkspace;

use super::PasteError;

#[derive(Clone)]
pub(crate) struct TargetApplication {
    pid: i32,
    bundle_id: Option<String>,
    display_name: String,
}

impl TargetApplication {
    pub(crate) fn display_name(&self) -> &str {
        &self.display_name
    }
}

pub(super) fn capture_target() -> Result<TargetApplication, String> {
    let application = NSWorkspace::sharedWorkspace()
        .frontmostApplication()
        .ok_or_else(|| "No frontmost application is available".to_string())?;
    let pid = application.processIdentifier();
    if pid <= 0 {
        return Err("The frontmost application has no process identity".into());
    }
    let bundle_id = application
        .bundleIdentifier()
        .map(|value| value.to_string());
    let display_name = application.localizedName().map_or_else(
        || bundle_id.clone().unwrap_or_else(|| format!("PID {pid}")),
        |value| value.to_string(),
    );
    Ok(TargetApplication {
        pid,
        bundle_id,
        display_name,
    })
}

pub(super) fn is_focused(expected: &TargetApplication) -> bool {
    capture_target()
        .is_ok_and(|current| current.pid == expected.pid && current.bundle_id == expected.bundle_id)
}

pub(super) fn has_permission() -> bool {
    unsafe { AXIsProcessTrusted() != 0 }
}

pub(super) fn permission_message() -> &'static str {
    "Knovum needs macOS Accessibility permission to insert text into another application"
}

pub(super) fn paste() -> Result<(), PasteError> {
    let source = CGEventSource::new(CGEventSourceStateID::HIDSystemState)
        .map_err(|()| PasteError::failed("Unable to create a macOS keyboard event source"))?;
    let down = CGEvent::new_keyboard_event(source.clone(), KeyCode::ANSI_V, true)
        .map_err(|()| PasteError::failed("Unable to create the paste key-down event"))?;
    let up = CGEvent::new_keyboard_event(source, KeyCode::ANSI_V, false)
        .map_err(|()| PasteError::failed("Unable to create the paste key-up event"))?;
    down.set_flags(CGEventFlags::CGEventFlagCommand);
    up.set_flags(CGEventFlags::CGEventFlagCommand);
    down.post(CGEventTapLocation::HID);
    up.post(CGEventTapLocation::HID);
    Ok(())
}

#[link(name = "ApplicationServices", kind = "framework")]
unsafe extern "C" {
    fn AXIsProcessTrusted() -> u8;
}
