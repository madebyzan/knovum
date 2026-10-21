pub(crate) mod browsing;
mod catalog;
mod client;
pub(super) mod commands;
pub(crate) mod connections;
pub(crate) mod endpoint;
pub(crate) mod http_client;
pub(crate) mod invocation;
pub(crate) mod resource_selection;
pub(crate) mod sessions;
mod stdio;
pub(crate) mod tool_schema;
pub(crate) use client::McpInteraction;

#[cfg(test)]
mod tests;
