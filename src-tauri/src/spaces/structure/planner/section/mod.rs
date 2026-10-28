mod draw;
mod flow;
mod frame;
mod layout;
mod wrap;

pub(in crate::spaces::structure) use draw::draw;
pub(in crate::spaces::structure) use flow::{require_freely_arranged, settle};
pub(in crate::spaces::structure) use frame::{dissolve, fit, resize};
pub(in crate::spaces::structure) use wrap::wrap;
