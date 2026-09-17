pub(crate) mod actions;
pub(crate) mod change_commit;
pub(crate) mod changes;
pub(crate) mod chunk_store;
pub(crate) mod contracts;
pub(crate) mod events;
pub(crate) mod grounding;
pub(crate) mod health;
pub(crate) mod indexing;
pub(crate) mod invalidation;
pub(crate) mod locator;
pub(crate) mod mcp;
mod node_contracts;
pub(crate) mod object_references;
pub(crate) mod provenance;
pub(crate) mod relations;
pub(crate) mod retrieval;
pub(crate) mod search_plan;
pub(crate) mod semantic_nodes;
pub(crate) mod types;

#[cfg(test)]
mod tests;
