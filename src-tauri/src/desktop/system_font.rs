/// The installed UI sans family used by generated local media and documents.
/// Renderers remain responsible for shaping, fallback, and file formats.
#[cfg(target_os = "macos")]
pub(crate) const SYSTEM_SANS_FAMILY: &str = "PingFang SC";

#[cfg(target_os = "windows")]
pub(crate) const SYSTEM_SANS_FAMILY: &str = "Microsoft YaHei UI";

// Linux verification installs this family through the fonts-noto-cjk package.
#[cfg(target_os = "linux")]
pub(crate) const SYSTEM_SANS_FAMILY: &str = "Noto Sans CJK SC";
