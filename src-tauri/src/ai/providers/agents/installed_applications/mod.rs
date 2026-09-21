#[cfg(target_os = "macos")]
mod macos;
#[cfg(windows)]
mod windows;

#[cfg(target_os = "macos")]
pub(super) fn detect() -> crate::ai::error::AiResult<std::collections::HashSet<crate::ai::contracts::AgentKind>> {
    macos::detect()
}

#[cfg(windows)]
pub(super) fn detect() -> crate::ai::error::AiResult<std::collections::HashSet<crate::ai::contracts::AgentKind>> {
    windows::detect()
}
