mod draw;
mod frame;
mod layout;
mod wrap;

pub(in crate::spaces::structure) use draw::draw;
pub(in crate::spaces::structure) use frame::{dissolve, fit, resize};
pub(in crate::spaces::structure) use wrap::wrap;
