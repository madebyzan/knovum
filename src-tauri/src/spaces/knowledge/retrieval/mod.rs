pub(crate) mod arrow_batch;
pub(crate) mod chunk_loader;
mod current_chunks;
mod generation;
mod generation_build;
mod global_search;
pub(crate) mod hybrid;
pub(crate) mod index_state;
pub(crate) mod index_writer;
pub(crate) mod indexing_control;
pub(crate) mod invalidation_outbox;
pub(crate) mod lancedb_store;
pub(crate) mod predicates;
pub(crate) mod rebuild;
pub(crate) mod related;
pub(crate) mod retired_generations;
pub(crate) mod schema;
pub(crate) mod search;
pub(crate) mod search_control;
mod search_projection;
mod validation;

pub(crate) use rebuild::index_space;

#[cfg(test)]
mod tests;
