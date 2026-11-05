//! Fixtures shared by the image-set suites. A set no feature publishes, so the shared machinery is
//! tested against the contract rather than against one feature's wording.

use super::ImageSet;

pub(super) const EXAMPLE: ImageSet = ImageSet {
    action_kind: "split-example",
    engine: "local:example",
    lineage: "Example member separated from source image",
    member: "part",
    members: 2..=24,
    unsupported: "This image did not separate into parts",
};
