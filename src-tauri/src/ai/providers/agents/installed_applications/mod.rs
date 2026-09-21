#[cfg(any(target_os = "macos", windows))]
use std::collections::HashSet;

#[cfg(any(target_os = "macos", windows))]
use crate::ai::{contracts::AgentKind, error::AiResult};

#[cfg(target_os = "macos")]
mod macos;
#[cfg(windows)]
mod windows;

#[cfg(target_os = "macos")]
pub(super) fn detect() -> AiResult<HashSet<AgentKind>> {
    macos::detect()
}

#[cfg(windows)]
pub(super) fn detect() -> AiResult<HashSet<AgentKind>> {
    windows::detect()
}
