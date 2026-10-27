//! Where desktop Agents work for a Space: one directory per Space conversation under the app's
//! data dir — the working directory of that conversation's Agent runtime — and the startup sweep
//! that drops every workspace no conversation owns.
pub(crate) mod recovery;
pub(crate) mod workspace;
