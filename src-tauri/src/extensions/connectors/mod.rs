pub(super) mod accounts;
pub(super) mod authorization;
pub(crate) mod cleanup;
pub(super) mod commands;
mod credential_store;
mod oauth_http;
pub(crate) mod requirements;
pub(crate) use authorization::credentials;

#[cfg(test)]
#[path = "tests/oauth.rs"]
mod oauth_tests;

#[cfg(test)]
#[path = "tests/accounts.rs"]
mod account_tests;
