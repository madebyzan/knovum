/// The installed UI sans family used by generated local media and documents.
/// Renderers remain responsible for shaping, fallback, and file formats.
#[cfg(target_os = "macos")]
pub(crate) const SYSTEM_SANS_FAMILY: &str = "PingFang SC";

#[cfg(target_os = "windows")]
pub(crate) const SYSTEM_SANS_FAMILY: &str = "Microsoft YaHei UI";
