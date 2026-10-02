pub(crate) mod arrow_batch;
mod current_chunks;
pub(crate) mod hybrid;
pub(crate) mod index_writer;
pub(crate) mod invalidation_outbox;
pub(crate) mod lancedb_store;
pub(crate) mod predicates;
pub(crate) mod rebuild;
pub(crate) mod record_loader;
pub(crate) mod related;
pub(crate) mod schema;
pub(crate) mod search;
pub(crate) mod search_control;
mod validation;

pub(crate) use rebuild::{index_space, rebuild_space_index};

#[cfg(test)]
mod tests;
