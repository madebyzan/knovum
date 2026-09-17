mod catalog;
pub mod chat;
mod connection;
pub mod discovery;
mod help;
#[cfg(target_os = "macos")]
#[path = "installed_applications_macos.rs"]
mod installed_applications;
#[cfg(windows)]
#[path = "installed_applications_windows.rs"]
mod installed_applications;
mod permissions;
pub mod prompt;
mod protocol;
pub mod registry;
pub mod session;
mod updates;

pub use discovery::detect_agent_installations;
