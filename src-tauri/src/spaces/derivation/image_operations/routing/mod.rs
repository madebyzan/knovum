mod target;
use crate::ai::{capabilities::image, models::ModelExecutionTarget};
pub(super) use target::local_capability;
pub(super) use target::{resolve, ResolvedImageTarget};

/// A remote image operation hands the model an image and an instruction, so the
/// model supports it exactly when it accepts a reference image.
pub(super) fn supports_remote_operations(target: &ModelExecutionTarget) -> bool {
    image::reference_support(target).is_some()
}

#[cfg(test)]
mod tests;
