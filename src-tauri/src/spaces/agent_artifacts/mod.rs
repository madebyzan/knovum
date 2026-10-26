//! Where desktop Agents work for a Space: one directory per External Agent conversation under the
//! app's data dir, and the startup sweep that drops every workspace no conversation owns.
pub(crate) mod recovery;
pub(crate) mod workspace;
