use crate::ai::capabilities::embedding::active_text_provider;

pub(super) fn active_model_id() -> &'static str {
    active_text_provider().model_id()
}

pub(super) fn active_dimensions() -> usize {
    active_text_provider().dimensions()
}

pub(super) fn active_batch_size() -> usize {
    active_text_provider().max_batch_size()
}

pub(super) fn format_query(query: &str) -> String {
    active_text_provider().format_query(query)
}
