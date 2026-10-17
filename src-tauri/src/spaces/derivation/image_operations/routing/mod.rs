mod target;
use crate::ai::{capabilities::image, models::ModelExecutionTarget};
pub(super) use target::local_capability;
pub(super) use target::{resolve, ResolvedImageTarget};

/// A remote image operation hands the model an image and an instruction, so the
/// model supports it exactly when it accepts a reference image.
pub(super) fn supports_remote_operations(target: &ModelExecutionTarget) -> bool {
    image::reference_budget(target).is_some_and(|(max_images, _)| max_images > 0)
}

#[cfg(test)]
mod tests;
