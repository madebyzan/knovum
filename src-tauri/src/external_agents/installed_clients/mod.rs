#[cfg(target_os = "macos")]
mod macos;
#[cfg(windows)]
mod windows;

#[cfg(target_os = "macos")]
pub(super) use macos::detect;
#[cfg(windows)]
pub(super) use windows::detect;
