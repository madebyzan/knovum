mod catalog;
mod connection;
pub mod conversation;
pub mod discovery;
mod help;
mod installed_clients;
pub(crate) mod permissions;
pub mod prompt;
mod protocol;
pub mod registry;
mod resource_links;
pub mod session;
pub(crate) mod task;
mod updates;

pub use discovery::detect_agent_installations;
