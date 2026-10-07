pub(crate) mod commands;
mod content_size;
pub(crate) mod contracts;
pub(crate) mod create_note_action;
mod derived;
mod insert;
mod lifecycle;
mod space_timestamp;
mod update;

pub(in crate::spaces) use content_size::fit_in_snapshot as fit_content_sizes_in_snapshot;
pub(in crate::spaces) use content_size::fitted_size as fitted_content_size;
pub(in crate::spaces) use content_size::sync_node_ids_in_transaction as sync_content_sizes_in_transaction;
pub use insert::insert_many;
pub(in crate::spaces) use insert::insert_many_in_transaction;
pub(in crate::spaces) use insert::insert_rows_in_transaction;
pub(in crate::spaces) use insert::insert_rows_with_actor;
pub(in crate::spaces) use lifecycle::restore_in_transaction;
pub(in crate::spaces) use lifecycle::trash_in_transaction;
pub(in crate::spaces) use space_timestamp::touch_updated_at;
pub(in crate::spaces) use update::update_many;
pub(in crate::spaces) use update::update_many_in_transaction;

#[cfg(test)]
mod tests;
