//! Where desktop Agents work for a Space: one directory per session under the app's data dir,
//! and the startup sweep that settles sessions a crash interrupted and drops workspaces of Spaces
//! that no longer exist.
pub(crate) mod recovery;
mod session;
pub(crate) mod workspace;
